#lang racket

;; Focused M3 operation canary.  This is intentionally a small, executable
;; contract: claim is pure Branch -> successor Branch, require is read-only
;; exact membership, and both operations emit host-neutral array JSON.

(require json
         racket/file
         racket/list
         racket/match
         racket/runtime-path
         racket/string
         "frontend.rkt"
         "kernel.rkt")

(unless (string=? (version) "9.3")
  (error 'm3-canary "Racket 9.3 is required; running ~a" (version)))

(define-runtime-path fixture-path "m3.clause")

(define (expect-failure thunk fragment)
  (with-handlers ([exn:fail?
                   (lambda (failure)
                     (unless (string-contains? (exn-message failure) fragment)
                       (raise failure))
                     #t)])
    (thunk)
    (error 'm3-canary "expected failure containing ~a" fragment)))

(define (literal-clause item)
  (Clause (parsed-item-name item)
          (make-immutable-hash
           (for/list ([role (in-list (parsed-item-roles item))])
             (cons (parsed-role-name role) (Literal (parsed-role-text role)))))))

(define (run-generated plan)
  (define directory (make-temporary-file "clause-m3-generated~a" 'directory))
  (define path (build-path directory "plan.rkt"))
  (dynamic-wind
    void
    (lambda ()
      (display-to-file (emit-racket-plan plan) path #:exists 'truncate)
      (define output
        (with-output-to-string
          (lambda ()
            (unless (system* (find-system-path 'exec-file) path)
              (error 'm3-canary "generated Racket plan failed")))))
      (string->jsexpr (string-trim output)))
    (lambda () (delete-directory/files directory))))

(define (expected-proof revision-id member)
  (list "proof"
        (format "proof/~a/catalog/contains/member=~a,set=letters" revision-id member)
        "relation" "catalog/contains"
        "roles" (list (list "member" member) (list "set" "letters"))))

(define (query-expected revision-id)
  (list "clause-query-output-v1"
        (list "results" '("a" "b" "c"))
        (list "proofs"
              (list (expected-proof revision-id "a")
                    (expected-proof revision-id "b")
                    (expected-proof revision-id "c")))))

(define (focused-canary)
  (define directory (make-temporary-file "clause-m3~a" 'directory))
  (define source-path (build-path directory "authoring.clause"))
  (define revision-path (build-path directory "revision.json"))
  (define post-claim-path (build-path directory "post-claim.json"))
  (dynamic-wind
    void
    (lambda ()
      (copy-file fixture-path source-path)
      (define elaborated (elaborate-source (file->string source-path)))

      ;; M3's two operations are trailing figures over the same exact sentence
      ;; shape.  The semantic payload is the canonical v3 form with no intents.
      (define operations (elaboration-operations elaborated))
      (unless (= (length operations) 2)
        (error 'm3-canary "expected one claim and one require operation"))
      (define claim-item (first operations))
      (define require-item (second operations))
      (unless (and (eq? (parsed-item-kind claim-item) 'claim)
                   (eq? (parsed-item-kind require-item) 'require)
                   (string=? (parsed-item-name claim-item) "catalog/contains")
                   (string=? (parsed-item-name require-item) "catalog/contains"))
        (error 'm3-canary "claim/require figures did not resolve to catalog/contains"))
      (define c (literal-clause claim-item))
      (unless (equal? c
                      (Clause "catalog/contains"
                              (hash "member" (Literal "c")
                                    "set" (Literal "letters"))))
        (error 'm3-canary "claim clause roles were not elaborated canonically"))
      (unless (equal? (literal-clause require-item) c)
        (error 'm3-canary "require clause differs from claim clause"))

      (define base-revision (admit-semantic (elaboration-semantic elaborated)))
      (define base-branch (Branch "catalog" base-revision))
      (define base-serialized (serialize-revision base-revision))
      (display-to-file base-serialized revision-path #:exists 'truncate)

      ;; A failed require is exact membership, not query planning or mutation.
      (define missing
        (require-clause (Branch-head base-branch) c))
      (define expected-missing
        (list "clause-require-output-v1" "unsatisfied"
              (list "revision" (Revision-identity base-revision))
              (list "clause" (list "clause" "catalog/contains" "roles"
                                      (list (list "member" (list "literal" "c"))
                                            (list "set" (list "literal" "letters")))))
              (list "diagnostic" "require.unsatisfied")))
      (unless (equal? (require-output->jsexpr missing) expected-missing)
        (error 'm3-canary "base require did not produce exact unsatisfied output"))

      ;; Claim is pure.  It must leave the input Branch and Revision byte-identical.
      (define-values (claimed-branch claimed-output) (claim base-branch c))
      (define next-revision (Branch-head claimed-branch))
      (unless (and (string=? (Branch-name claimed-branch) "catalog")
                   (not (string=? (Revision-identity next-revision)
                                  (Revision-identity base-revision)))
                   (equal? (serialize-revision (Branch-head base-branch)) base-serialized))
        (error 'm3-canary "claim mutated or replaced its input branch"))
      (define expected-claim
        (list "clause-claim-output-v1" "admitted"
              (list "branch" "catalog")
              (list "base" (Revision-identity base-revision))
              (list "revision" (Revision-identity next-revision))
              (list "fact" (list "fact" "catalog/contains" "roles"
                                  (list (list "member" (list "literal" "c"))
                                        (list "set" (list "literal" "letters")))))))
      (unless (equal? (claim-output->jsexpr claimed-output) expected-claim)
        (error 'm3-canary "claim output drifted from clause-claim-output-v1"))

      ;; Duplicate claim is deterministic and does not create a third revision.
      ;; It is attempted against the already-admitted successor, so its current
      ;; revision (NEXT) is the duplicate diagnostic's revision field.
      (define-values (duplicate-branch duplicate-output) (claim claimed-branch c))
      (unless (and (equal? duplicate-branch claimed-branch)
                   (equal? (serialize-revision (Branch-head duplicate-branch))
                           (serialize-revision next-revision))
                   (equal? (claim-output->jsexpr duplicate-output)
                           (list "clause-claim-output-v1" "duplicate"
                                 (list "branch" "catalog")
                                 (list "revision" (Revision-identity next-revision))
                                 (list "diagnostic" "claim.duplicate"))))
        (error 'm3-canary "duplicate claim was not a deterministic no-op"))

      ;; The admitted successor satisfies the same closed require exactly.
      (define satisfied
        (require-clause next-revision c))
      (define expected-satisfied
        (list "clause-require-output-v1" "satisfied"
              (list "revision" (Revision-identity next-revision))
              (list "proof" (expected-proof (Revision-identity next-revision) "c"))))
      (unless (equal? (require-output->jsexpr satisfied) expected-satisfied)
        (error 'm3-canary "post-claim require did not produce exact proof"))

      ;; M2 query/proof semantics continue over the fresh immutable revision.
      (define post-plan (check-query next-revision))
      (define interpreted (interpret-plan post-plan))
      (unless (and (equal? (QueryOutput-results interpreted) '("a" "b" "c"))
                   (equal? (query-output->jsexpr interpreted)
                           (query-expected (Revision-identity next-revision)))
                   (andmap (lambda (proof) (proof-valid? next-revision proof))
                           (QueryOutput-proofs interpreted)))
        (error 'm3-canary "post-claim query/proof output drifted"))

      ;; Source deletion, strict reload, and tamper rejection hold after claim.
      (define post-serialized (serialize-revision next-revision))
      (display-to-file post-serialized post-claim-path #:exists 'truncate)
      (delete-file source-path)
      (unless (equal? (Revision-identity (reload-revision (file->string post-claim-path)))
                      (Revision-identity next-revision))
        (error 'm3-canary "post-claim revision did not reload after source deletion"))
      (expect-failure
       (lambda ()
         (reload-revision
          (jsexpr->string
           (list "clause-revision-v1" "rev-sha256-tampered"
                 (third (string->jsexpr post-serialized))))))
       "identity")
      (define post-envelope (string->jsexpr post-serialized))
      (define broken-semantic
        (match (third post-envelope)
          [(list version relations facts (list "query" query) intents order)
           (list version relations facts
                 (list "query"
                       (match query
                         [(list kind relation "roles" roles)
                          (list kind relation "roles" (rest roles))]))
                 intents order)]))
      (expect-failure
       (lambda ()
         (reload-revision
          (jsexpr->string (list "clause-revision-v1"
                                 (Revision-identity next-revision)
                                 broken-semantic))))
       "role map")

      ;; Generated host code and the interpreter must agree byte-for-byte.
      (unless (equal? (query-output->jsexpr interpreted)
                      (run-generated post-plan))
        (error 'm3-canary "generated Racket output diverged from interpreter"))

      (displayln (jsexpr->string (claim-output->jsexpr claimed-output)))
      (displayln (jsexpr->string (require-output->jsexpr satisfied)))
      (displayln (jsexpr->string (query-output->jsexpr interpreted)))
      (displayln (format "m3 canary: claim/require, duplicate no-op, source deletion/reload/tamper, generated-host parity (~a -> ~a)"
                         (Revision-identity base-revision)
                         (Revision-identity next-revision))))
    (lambda () (delete-directory/files directory))))

(module+ main
  (match (vector->list (current-command-line-arguments))
    [(list "--canary") (focused-canary)]
    [_ (error 'm3-canary "use --canary")]))
