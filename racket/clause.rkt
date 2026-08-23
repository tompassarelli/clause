#lang racket

(require json
         racket/file
         racket/match
         racket/path
         racket/port
         racket/runtime-path
         racket/string
         "frontend.rkt"
         "kernel.rkt")

(unless (string=? (version) "9.3")
  (error 'clause "Racket 9.3 is required; running ~a" (version)))

(define-runtime-path fixture-path "canary.clause")

(define (normalized value)
  (cond [(hash? value)
         (for/list ([key (in-list (sort (hash-keys value) string<? #:key symbol->string))])
           (list (symbol->string key) (normalized (hash-ref value key))))]
        [(list? value) (map normalized value)]
        [else value]))

(define (expect-reload-failure text fragment)
  (with-handlers ([exn:fail? (lambda (failure)
                               (unless (string-contains? (exn-message failure) fragment)
                                 (raise failure))
                               #t)])
    (reload-revision text)
    (error 'canary "tampered revision was admitted")))

(define (focused-canary)
  (define directory (make-temporary-file "clause-racket~a" 'directory))
  (define source-path (build-path directory "authoring.clause"))
  (define revision-path (build-path directory "revision.json"))
  (define program-path (build-path directory "checked-plan.rkt"))
  (dynamic-wind
    void
    (lambda ()
      (copy-file fixture-path source-path)
      (define elaborated (elaborate-source (file->string source-path)))
      (unless (and (= (length (elaboration-value-spans elaborated)) 6)
                   (andmap positive? (map source-span-width (elaboration-value-spans elaborated)))
                   (equal? (map source-span-column (elaboration-value-spans elaborated))
                           '(5 23 5 23 19 38))
                   (= (source-span-line (last (elaboration-value-spans elaborated))) 10))
        (error 'canary "complete named-role value spans were not preserved through elaboration"))
      (define admitted (admit-semantic (elaboration-semantic elaborated)))
      (define serialized (serialize-revision admitted))
      (display-to-file serialized revision-path #:exists 'truncate)
      (delete-file source-path)
      (when (file-exists? source-path) (error 'canary "authoring source was not deleted"))
      (define reloaded-a (reload-revision (file->string revision-path)))
      (define reloaded-b (reload-revision (file->string revision-path)))
      (define reloaded-model (Revision-model reloaded-a))
      (unless (and (equal? reloaded-a reloaded-b)
                   (immutable? (Model-relations reloaded-model))
                   (andmap (lambda (relation) (immutable? (Relation-roles relation)))
                           (hash-values (Model-relations reloaded-model)))
                   (andmap (lambda (fact) (immutable? (Clause-roles fact)))
                           (Model-facts reloaded-model))
                   (immutable? (Clause-roles (Model-query reloaded-model)))
                   (immutable? (Revision-identity reloaded-a))
                   (immutable? (Clause-relation (Model-query reloaded-model)))
                   (immutable? (Literal-text
                                (hash-ref (Clause-roles (Model-query reloaded-model)) "set"))))
        (error 'canary "reload was not deterministic and immutable"))
      (define envelope (string->jsexpr serialized))
      (define semantic (third envelope))
      (unless (and (= (length envelope) 3)
                   (equal? envelope
                           (list "clause-revision-v1"
                                 (Revision-identity admitted)
                                 (elaboration-semantic elaborated))))
        (error 'canary "revision wire envelope or canonical semantic payload drifted"))
      (expect-reload-failure (jsexpr->string (list (first envelope) "rev-sha256-tampered" (third envelope)))
                             "identity")
      (expect-reload-failure (string-append " " serialized) "canonical")
      (define broken-query
        (match semantic
          [(list version relations facts (list "query" (list kind relation "roles" roles)) intents order)
           (list version relations facts (list "query" (list kind relation "roles" (rest roles))) intents order)]))
      (expect-reload-failure (jsexpr->string (list (first envelope) (second envelope) broken-query))
                             "role map")
      (define plan (check-query reloaded-a))
      (define interpreted-a (interpret-plan plan))
      (define interpreted-b (interpret-plan (check-query reloaded-b)))
      (unless (and (equal? interpreted-a interpreted-b)
                   (equal? (CheckedPlan-mode plan) (mode 'finite '("set") '("member") 'many))
                   (equal? (QueryOutput-results interpreted-a) '("a" "b"))
                   (andmap (lambda (proof) (proof-valid? reloaded-a proof))
                           (QueryOutput-proofs interpreted-a)))
        (error 'canary "reload/query/proof output was not deterministic or valid"))
      (define revision-id (Revision-identity reloaded-a))
      (define expected-output
        (list "clause-query-output-v1"
              (list "results" '("a" "b"))
              (list "proofs"
                    (list
                     (list "proof"
                           (format "proof/~a/catalog/contains/member=a,set=letters" revision-id)
                           "relation" "catalog/contains"
                           "roles" '(("member" "a") ("set" "letters")))
                     (list "proof"
                           (format "proof/~a/catalog/contains/member=b,set=letters" revision-id)
                           "relation" "catalog/contains"
                           "roles" '(("member" "b") ("set" "letters")))))))
      (unless (equal? (query-output->jsexpr interpreted-a) expected-output)
        (error 'canary "query-output-v1 wire shape or proof identities drifted"))
      (display-to-file (emit-racket-plan plan) program-path #:exists 'truncate)
      (define racket-output
        (string-trim
         (with-output-to-string
           (lambda ()
             (unless (system* (find-system-path 'exec-file) program-path)
               (error 'canary "Racket CS did not execute the generated plan"))))))
      (unless (equal? (normalized (query-output->jsexpr interpreted-a))
                      (normalized (string->jsexpr racket-output)))
        (error 'canary "generated Racket result/proofs diverged from interpreter"))
      (displayln (jsexpr->string (query-output->jsexpr interpreted-a)))
      (displayln (format "canary: source deleted; strict reload/tamper checks; deterministic interpreter/generated-Racket parity (~a)"
                         (Revision-identity reloaded-a))))
    (lambda () (delete-directory/files directory))))

(module+ main
  (match (vector->list (current-command-line-arguments))
    [(list "--canary") (focused-canary)]
    [_ (error 'clause "use --canary")]))
