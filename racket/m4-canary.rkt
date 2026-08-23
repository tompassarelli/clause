#lang racket

;; M4 end-to-end canary.  The three M4 seam APIs named below are deliberately
;; the join contract for the frontend, kernel/operations, and e2e generator:
;;
;;   parse-source       : String -> (listof parsed-item), including intents
;;   intent              : Branch String -> IntentOutput
;;   emit-racket-e2e     : String String -> String
;;
;; The parser and operation names are the joined frontend/kernel/operations
;; contract.  The emitter below is canary-owned and produces a self-contained
;; host from persisted revision bytes and an intent name.

(require json
         racket/file
         racket/list
         racket/match
         racket/runtime-path
         racket/string
         "frontend.rkt"
         "kernel.rkt"
         "intent-operations.rkt")

(provide emit-racket-e2e)

(define-runtime-path fixture-path "m4.clause")

(define BASE "rev-sha256-746240d8119edb45ce1971043d46fa865847efa799b682463d484445aa7b8f77")
(define NEXT "rev-sha256-aa2dc7de2b7489b035a4cd6194f2b436a89c765d462eb055d5a01ffdd2004ceb")
(define INTENT-NAME "catalog/restock")

(define (expect-failure thunk fragment)
  (with-handlers ([exn:fail?
                   (lambda (failure)
                     (unless (string-contains? (exn-message failure) fragment)
                       (raise failure))
                     #t)])
    (thunk)
    (error 'm4-canary "expected failure containing ~a" fragment)))

(define (literal-clause item relation-name)
  (Clause relation-name
          (make-immutable-hash
           (for/list ([role (in-list (parsed-item-roles item))])
             (cons (parsed-role-name role) (Literal (parsed-role-text role)))))))

(define (clause-datum kind clause)
  (list kind (Clause-relation clause) "roles"
        (for/list ([name (in-list (sort (hash-keys (Clause-roles clause)) string<?))])
          (define term (hash-ref (Clause-roles clause) name))
          (list name
                (if (Literal? term)
                    (list "literal" (Literal-text term))
                    (list "variable" (Variable-name term)))))))

(define DESIRED
  (list "clause" "catalog/contains" "roles"
        (list (list "member" (list "literal" "c"))
              (list "set" (list "literal" "letters")))))

(define DESIRED-FACT
  (list "fact" "catalog/contains" "roles"
        (list (list "member" (list "literal" "c"))
              (list "set" (list "literal" "letters")))))

(define (expected-proof revision-id member)
  (list "proof"
        (format "proof/~a/catalog/contains/member=~a,set=letters" revision-id member)
        "relation" "catalog/contains"
        "roles" (list (list "member" member) (list "set" "letters"))))

(define (expected-query revision-id members)
  (list "clause-query-output-v1"
        (list "results" members)
        (list "proofs"
              (for/list ([member (in-list members)])
                (expected-proof revision-id member)))))

(define (expected-proposed-intent revision-id)
  (list "clause-intent-output-v1" "proposed"
        (list "revision" revision-id)
        (list "intent" INTENT-NAME)
        (list "desired" DESIRED)
        (list "plan"
              (list "plan" (format "plan/~a/~a" revision-id INTENT-NAME)
                    "operation" "claim"
                    "base" revision-id
                    "fact" DESIRED-FACT))
        (list "explanation"
              (list "explanation" "desired-clause-is-absent"
                    "revision" revision-id
                    "clause" DESIRED
                    "diagnostic" "require.unsatisfied"))))

(define (expected-already-satisfied revision-id)
  (list "clause-intent-output-v1" "already-satisfied"
        (list "revision" revision-id)
        (list "intent" INTENT-NAME)
        (list "desired" DESIRED)
        (list "proof" (expected-proof revision-id "c"))
        (list "explanation"
              (list "explanation" "desired-clause-is-claimed"
                    "revision" revision-id))))

(define (expected-claim base-id next-id)
  (list "clause-claim-output-v1" "admitted"
        (list "branch" "catalog")
        (list "base" base-id)
        (list "revision" next-id)
        (list "fact" DESIRED-FACT)))

(define (expected-require revision-id)
  (list "clause-require-output-v1" "satisfied"
        (list "revision" revision-id)
        (list "proof" (expected-proof revision-id "c"))))

(define (expected-e2e base-query base-intent admitted satisfied next-query next-intent)
  (list "clause-e2e-output-v1"
        base-query
        base-intent
        admitted
        satisfied
        next-query
        next-intent))

(define (model-strings-immutable? model)
  (define (term-immutable? term)
    (if (Literal? term)
        (immutable? (Literal-text term))
        (immutable? (Variable-name term))))
  (define (clause-immutable? clause)
    (and (immutable? (Clause-relation clause))
         (andmap immutable? (hash-keys (Clause-roles clause)))
         (andmap term-immutable? (hash-values (Clause-roles clause)))))
  (and (andmap immutable? (hash-keys (Model-relations model)))
       (andmap
        (lambda (relation)
          (and (immutable? (Relation-name relation))
               (andmap immutable? (hash-keys (Relation-roles relation)))
               (andmap (lambda (role)
                         (and (immutable? (Role-name role))
                              (immutable? (Role-type role))))
                       (hash-values (Relation-roles relation)))
               (andmap immutable? (Sentence-roles (Relation-sentence relation)))
               (immutable? (Sentence-literal (Relation-sentence relation)))
               (andmap (lambda (declared-mode)
                         (andmap immutable? (mode-known declared-mode)))
                       (Relation-modes relation))
               (andmap (lambda (declared-mode)
                         (andmap immutable? (mode-sought declared-mode)))
                       (Relation-modes relation))))
        (hash-values (Model-relations model)))
       (andmap clause-immutable? (Model-facts model))
       (clause-immutable? (Model-query model))
       (andmap (lambda (item)
                 (and (immutable? (Intent-name item))
                      (clause-immutable? (Intent-desired item))))
               (Model-intents model))))

(define (semantic-with-intent semantic intent-entry)
  (list (list-ref semantic 0)
        (list-ref semantic 1)
        (list-ref semantic 2)
        (list-ref semantic 3)
        (list "intents" (list intent-entry))
        (list-ref semantic 5)))

(define (semantic-with-facts semantic facts)
  (list (list-ref semantic 0)
        (list-ref semantic 1)
        (list "facts" facts)
        (list-ref semantic 3)
        (list-ref semantic 4)
        (list-ref semantic 5)))

(define (revision-with-semantic envelope semantic)
  (jsexpr->string
   (list "clause-revision-v1" (list-ref envelope 1) semantic)))

(define (emit-racket-e2e serialized-base intent-name)
  (string-append
   "#lang racket/base\n"
   "(require json racket/list racket/string)\n"
   (format "(define persisted-base ~s)\n" serialized-base)
   (format "(define selected-intent ~s)\n" intent-name)
   "(define (bytes->hex data)\n"
   "  (apply string-append\n"
   "         (for/list ([byte (in-bytes data)])\n"
   "           (define digits (number->string byte 16))\n"
   "           (if (= (string-length digits) 1) (string-append \"0\" digits) digits))))\n"
   "(define (revision-id semantic)\n"
   "  (string-append \"rev-sha256-\"\n"
   "                 (bytes->hex\n"
   "                  (sha256-bytes\n"
   "                   (open-input-bytes\n"
   "                    (string->bytes/utf-8 (jsexpr->string semantic)))))))\n"
   "(define (reload-revision persisted)\n"
   "  (define envelope (string->jsexpr persisted))\n"
   "  (unless (string=? persisted (jsexpr->string envelope))\n"
   "    (error 'generated-e2e \"revision envelope is not canonical\"))\n"
   "  (unless (and (list? envelope) (= (length envelope) 3)\n"
   "               (string=? (first envelope) \"clause-revision-v1\")\n"
   "               (string? (second envelope)))\n"
   "    (error 'generated-e2e \"invalid revision envelope\"))\n"
   "  (define semantic (third envelope))\n"
   "  (define computed-id (revision-id semantic))\n"
   "  (unless (string=? (second envelope) computed-id)\n"
   "    (error 'generated-e2e \"revision identity does not match semantic payload\"))\n"
   "  (values computed-id semantic))\n"
   "(define (section semantic name)\n"
   "  (second (findf (lambda (entry) (string=? (first entry) name)) (rest semantic))))\n"
   "(define-values (base-id semantic) (reload-revision persisted-base))\n"
   "(define facts (section semantic \"facts\"))\n"
   "(define query (section semantic \"query\"))\n"
   "(define intent-entry\n"
   "  (findf (lambda (entry) (string=? (second entry) selected-intent))\n"
   "         (section semantic \"intents\")))\n"
   "(unless intent-entry (error 'generated-e2e \"intent.unknown\"))\n"
   "(define desired (fourth intent-entry))\n"
   "(define (role-term roles name)\n"
   "  (or (assoc name roles) (error 'generated-e2e \"missing role\")))\n"
   "(define (role-value roles name) (second (second (role-term roles name))))\n"
   "(define (clause-roles clause) (fourth clause))\n"
   "(define (clause-value clause name) (role-value (clause-roles clause) name))\n"
   "(define query-roles (clause-roles query))\n"
   "(define variable-entry\n"
   "  (findf (lambda (entry) (string=? (first (second entry)) \"variable\")) query-roles))\n"
   "(unless variable-entry (error 'generated-e2e \"query has no variable\"))\n"
   "(define variable-role (first variable-entry))\n"
   "(define branch-name (first (string-split (second query) \"/\")))\n"
   "(define (proof revision-id clause)\n"
   "  (define roles (clause-roles clause))\n"
   "  (list \"proof\"\n"
   "        (string-append \"proof/\" revision-id \"/\" (second clause) \"/\"\n"
   "                       (string-join\n"
   "                        (map (lambda (entry)\n"
   "                               (format \"~a=~a\" (first entry) (role-value roles (first entry))))\n"
   "                             roles)\n"
   "                        \",\"))\n"
   "        \"relation\" (second clause)\n"
   "        \"roles\" (map (lambda (entry)\n"
   "                        (list (first entry) (role-value roles (first entry))))\n"
   "                      roles)))\n"
   "(define (matches-query? fact)\n"
   "  (and (string=? (second fact) (second query))\n"
   "       (for/and ([entry query-roles])\n"
   "         (or (string=? (first (second entry)) \"variable\")\n"
   "             (equal? (second entry)\n"
   "                     (second (role-term (clause-roles fact) (first entry))))))))\n"
   "(define (query-output revision-id query-facts)\n"
   "  (define matches (filter matches-query? query-facts))\n"
   "  (define ordered\n"
   "    (sort matches string<? #:key (lambda (fact) (clause-value fact variable-role))))\n"
   "  (list \"clause-query-output-v1\"\n"
   "        (list \"results\" (map (lambda (fact) (clause-value fact variable-role)) ordered))\n"
   "        (list \"proofs\" (map (lambda (fact) (proof revision-id fact)) ordered))))\n"
   "(define desired-fact (list \"fact\" (second desired) \"roles\" (fourth desired)))\n"
   "(define (next-semantic)\n"
   "  (list (first semantic)\n"
   "        (list-ref semantic 1)\n"
   "        (list \"facts\"\n"
   "              (sort (append facts (list desired-fact)) string<? #:key jsexpr->string))\n"
   "        (list-ref semantic 3)\n"
   "        (list-ref semantic 4)\n"
   "        (list-ref semantic 5)))\n"
   "(define next-serialized\n"
   "  (let ([next (next-semantic)])\n"
   "    (jsexpr->string (list \"clause-revision-v1\" (revision-id next) next))))\n"
   "(define-values (next-id reloaded-next-semantic) (reload-revision next-serialized))\n"
   "(define (intent-output revision-id revision-facts)\n"
   "  (define claimed? (ormap (lambda (fact) (equal? fact desired-fact)) revision-facts))\n"
   "  (if claimed?\n"
   "      (list \"clause-intent-output-v1\" \"already-satisfied\"\n"
   "            (list \"revision\" revision-id)\n"
   "            (list \"intent\" selected-intent)\n"
   "            (list \"desired\" desired)\n"
   "            (list \"proof\" (proof revision-id desired-fact))\n"
   "            (list \"explanation\"\n"
   "                  (list \"explanation\" \"desired-clause-is-claimed\"\n"
   "                        \"revision\" revision-id)))\n"
   "      (list \"clause-intent-output-v1\" \"proposed\"\n"
   "            (list \"revision\" revision-id)\n"
   "            (list \"intent\" selected-intent)\n"
   "            (list \"desired\" desired)\n"
   "            (list \"plan\"\n"
   "                  (list \"plan\" (string-append \"plan/\" revision-id \"/\" selected-intent)\n"
   "                        \"operation\" \"claim\" \"base\" revision-id\n"
   "                        \"fact\" desired-fact))\n"
   "            (list \"explanation\"\n"
   "                  (list \"explanation\" \"desired-clause-is-absent\"\n"
   "                        \"revision\" revision-id \"clause\" desired\n"
   "                        \"diagnostic\" \"require.unsatisfied\")))))\n"
   "(define base-query (query-output base-id facts))\n"
   "(define base-intent (intent-output base-id facts))\n"
   "(define next-facts\n"
   "  (section reloaded-next-semantic \"facts\"))\n"
   "(define admitted\n"
   "  (list \"clause-claim-output-v1\" \"admitted\"\n"
   "        (list \"branch\" branch-name) (list \"base\" base-id)\n"
   "        (list \"revision\" next-id) (list \"fact\" desired-fact)))\n"
   "(define satisfied\n"
   "  (list \"clause-require-output-v1\" \"satisfied\"\n"
   "        (list \"revision\" next-id)\n"
   "        (list \"proof\" (proof next-id desired-fact))))\n"
   "(define next-query (query-output next-id next-facts))\n"
   "(define next-intent (intent-output next-id next-facts))\n"
   "(displayln\n"
   " (jsexpr->string\n"
   "  (list \"clause-e2e-output-v1\"\n"
   "        base-query base-intent admitted satisfied next-query next-intent)))\n"))

(define (run-generated program)
  (define directory (make-temporary-file "clause-m4-generated~a" 'directory))
  (define path (build-path directory "e2e.rkt"))
  (dynamic-wind
    void
    (lambda ()
      (display-to-file program path #:exists 'truncate)
      (define output
        (with-output-to-string
          (lambda ()
            (unless (system* (find-system-path 'exec-file) path)
              (error 'm4-canary "generated Racket e2e program failed")))))
      (string->jsexpr (string-trim output)))
    (lambda () (delete-directory/files directory))))

(define renamed-source
  (string-append
   "relation orchard/harvest(crate: Text, fruit: Text):\n"
   "    sentence: {crate} yields {fruit}\n"
   "    mode crate -> fruit: many\n\n"
   "model orchard:\n"
   "    \"summer\" yields \"plum\"\n"
   "    \"summer\" yields \"apricot\"\n\n"
   "intent orchard/replenish:\n"
   "    \"summer\" yields \"pear\"\n\n"
   "query orchard:\n"
   "    ?fruit where \"summer\" yields ?fruit\n"))

(define (renamed-generated-canary)
  (define directory (make-temporary-file "clause-m4-renamed~a" 'directory))
  (define source-path (build-path directory "authoring.clause"))
  (define revision-path (build-path directory "revision.json"))
  (dynamic-wind
    void
    (lambda ()
      (display-to-file renamed-source source-path #:exists 'truncate)
      (define source (file->string source-path))
      (define parsed (parse-source source))
      (define elaborated (elaborate-source source))
      (define desired (literal-clause (first (third parsed))
                                      (parsed-item-name (first parsed))))
      (define base-revision (admit-semantic (elaboration-semantic elaborated)))
      (define serialized-base (serialize-revision base-revision))
      (display-to-file serialized-base revision-path #:exists 'truncate)
      (delete-file source-path)
      (define reloaded-base (reload-revision (file->string revision-path)))
      (unless (and (not (file-exists? source-path))
                   (equal? reloaded-base base-revision)
                   (equal? (QueryOutput-results
                            (interpret-plan (check-query reloaded-base)))
                           '("apricot" "plum")))
        (error 'm4-canary "renamed persisted revision did not survive source deletion"))
      (define base-branch (Branch "orchard" reloaded-base))
      (define base-query
        (query-output->jsexpr (interpret-plan (check-query reloaded-base))))
      (define base-intent-output (intent base-branch "orchard/replenish"))
      (define-values (claimed-branch claim-output) (claim base-branch desired))
      (define next-revision (Branch-head claimed-branch))
      (define require-output (require-clause next-revision desired))
      (define next-query
        (query-output->jsexpr (interpret-plan (check-query next-revision))))
      (define next-intent-output (intent claimed-branch "orchard/replenish"))
      (define interpreted
        (expected-e2e base-query
                      (intent-output->jsexpr base-intent-output)
                      (claim-output->jsexpr claim-output)
                      (require-output->jsexpr require-output)
                      next-query
                      (intent-output->jsexpr next-intent-output)))
      (define generated
        (run-generated (emit-racket-e2e serialized-base "orchard/replenish")))
      (unless (and (equal? generated interpreted)
                   (equal? (QueryOutput-results (interpret-plan (check-query next-revision)))
                           '("apricot" "pear" "plum")))
        (error 'm4-canary "renamed generated execution diverged from persisted interpreter revision"))
      (displayln "m4 renamed canary: orchard/harvest crate/fruit persisted source-deletion parity"))
    (lambda () (delete-directory/files directory))))

(define (focused-canary)
  (define directory (make-temporary-file "clause-m4~a" 'directory))
  (define source-path (build-path directory "authoring.clause"))
  (define revision-path (build-path directory "revision.json"))
  (dynamic-wind
    void
    (lambda ()
      (copy-file fixture-path source-path)
      (define source (file->string source-path))
      (define parsed (parse-source source))
      (define elaborated (elaborate-source source))

      ;; The frontend must retain the single typed, closed intent item beside
      ;; the unchanged M3 elaboration surface.
      (define intents (third parsed))
      (unless (and (= (length intents) 1)
                   (eq? (parsed-item-kind (first intents)) 'intent)
                   (string=? (parsed-item-name (first intents)) INTENT-NAME))
        (error 'm4-canary "M4 frontend did not elaborate the one named intent"))
      (define desired
        (literal-clause (first intents) (parsed-item-name (first parsed))))
      (unless (equal? (clause-datum "clause" desired) DESIRED)
        (error 'm4-canary "intent desired clause was not elaborated canonically"))

      ;; The v3 semantic payload, canonical bytes, and base identity are sealed.
      (define semantic (elaboration-semantic elaborated))
      (define expected-semantic
        (list "clause-semantic-v3"
              (list "relations"
                    (list (list "relation" "catalog/contains"
                                "roles" (list (list "member" "Text") (list "set" "Text"))
                                "sentence" (list "set" "contains" "member")
                                "modes" (list (list "mode" "finite"
                                                    "known" (list "set")
                                                    "sought" (list "member")
                                                    "cardinality" "many")))))
              (list "facts"
                    (list (list "fact" "catalog/contains" "roles"
                                (list (list "member" (list "literal" "a"))
                                      (list "set" (list "literal" "letters"))))
                          (list "fact" "catalog/contains" "roles"
                                (list (list "member" (list "literal" "b"))
                                      (list "set" (list "literal" "letters"))))))
              (list "query" (list "query" "catalog/contains" "roles"
                                    (list (list "member" (list "variable" "member"))
                                          (list "set" (list "literal" "letters")))))
              (list "intents"
                    (list (list "intent" INTENT-NAME "desired" DESIRED)))
              (list "order" "ascending" "member")))
      (unless (equal? semantic expected-semantic)
        (error 'm4-canary "frontend semantic payload drifted from clause-semantic-v3"))
      (define semantic-bytes (jsexpr->string semantic))
      (unless (and (string=? semantic-bytes (jsexpr->string expected-semantic))
                   (not (regexp-match? #px"\\s" semantic-bytes)))
        (error 'm4-canary "semantic bytes contain spaces or drifted"))

      (define base-revision (admit-semantic semantic))
      (unless (string=? (Revision-identity base-revision) BASE)
        (error 'm4-canary "sealed base revision identity changed"))
      (define base-branch (Branch "catalog" base-revision))
      (define serialized-base (serialize-revision base-revision))
      (display-to-file serialized-base revision-path #:exists 'truncate)
      (define generated-program (emit-racket-e2e serialized-base INTENT-NAME))

      ;; Source deletion is part of the proof boundary: all later work uses
      ;; only the persisted, strictly reloaded revision.
      (delete-file source-path)
      (unless (not (file-exists? source-path))
        (error 'm4-canary "authoring source was not deleted"))
      (define reloaded-base (reload-revision (file->string revision-path)))
      (unless (and (equal? reloaded-base base-revision)
                   (string=? (Revision-identity reloaded-base) BASE)
                   (immutable? (Revision-identity reloaded-base))
                   (model-strings-immutable? (Revision-model reloaded-base)))
        (error 'm4-canary "base revision did not strictly reload"))

      (define base-query
        (query-output->jsexpr (interpret-plan (check-query reloaded-base))))
      (unless (equal? base-query (expected-query BASE '("a" "b")))
        (error 'm4-canary "base query output drifted"))

      ;; intent is read-only and returns only a proposed claim plan.
      (define base-intent-output (intent base-branch INTENT-NAME))
      (unless (and (equal? base-branch (Branch "catalog" base-revision))
                   (equal? (intent-output->jsexpr base-intent-output)
                           (expected-proposed-intent BASE)))
        (error 'm4-canary "base intent was not an exact pure proposal"))
      (unless (equal? (intent-output->jsexpr (intent base-branch "catalog/missing"))
                      (list "clause-intent-output-v1" "rejected"
                            (list "revision" BASE)
                            (list "intent" "catalog/missing")
                            (list "diagnostic" "intent.unknown")))
        (error 'm4-canary "unknown intent selection drifted"))

      ;; The only state-changing operation is the existing explicit M3 claim.
      (define-values (claimed-branch claim-output) (claim base-branch desired))
      (define next-revision (Branch-head claimed-branch))
      (unless (and (string=? (Revision-identity next-revision) NEXT)
                   (equal? (claim-output->jsexpr claim-output)
                           (expected-claim BASE NEXT))
                   (equal? (serialize-revision (Branch-head base-branch))
                           serialized-base))
        (error 'm4-canary "claim did not admit the sealed next revision purely"))

      (define require-output (require-clause next-revision desired))
      (unless (equal? (require-output->jsexpr require-output)
                      (expected-require NEXT))
        (error 'm4-canary "post-claim require proof drifted"))
      (define next-query
        (query-output->jsexpr (interpret-plan (check-query next-revision))))
      (unless (equal? next-query (expected-query NEXT '("a" "b" "c")))
        (error 'm4-canary "next query output drifted"))
      (define next-intent-output (intent claimed-branch INTENT-NAME))
      (unless (equal? (intent-output->jsexpr next-intent-output)
                      (expected-already-satisfied NEXT))
        (error 'm4-canary "already-satisfied intent output drifted"))

      ;; Tampering is rejected by reload before any operation is evaluated.
      (define envelope (string->jsexpr serialized-base))
      (define intent-entry
        (list "intent" "catalog/restock" "desired" DESIRED))
      (expect-failure
       (lambda ()
         (reload-revision
          (revision-with-semantic envelope
                                  (semantic-with-intent
                                   (third envelope)
                                   (list "intent" "catalog/tampered" "desired" DESIRED)))))
       "identity")
      (expect-failure
       (lambda ()
         (reload-revision
          (revision-with-semantic envelope
                                  (semantic-with-intent
                                   (third envelope)
                                   (list "intent" INTENT-NAME "desired"
                                         (list "clause" "catalog/contains" "roles"
                                               (list (list "member" (list "literal" "d"))
                                                     (list "set" (list "literal" "letters")))))))))
       "identity")
      (expect-failure
       (lambda ()
         (reload-revision
          (revision-with-semantic envelope
                                  (semantic-with-facts
                                   (third envelope)
                                   (reverse (second (list-ref (third envelope) 2)))))))
       "canonical")
      (expect-failure
       (lambda ()
         (reload-revision
          (jsexpr->string
           (list "clause-revision-v1" "rev-sha256-tampered" (third envelope)))))
       "identity")
      (unless (equal? intent-entry
                      (first (second (list-ref (third envelope) 4))))
        (error 'm4-canary "base intent wire entry changed before tamper checks"))

      ;; The generated host independently reloads the persisted base and must
      ;; emit the exact same six-entry byte sequence as the interpreter.
      (define interpreted
        (expected-e2e base-query
                      (intent-output->jsexpr base-intent-output)
                      (claim-output->jsexpr claim-output)
                      (require-output->jsexpr require-output)
                      next-query
                      (intent-output->jsexpr next-intent-output)))
      (define generated (run-generated generated-program))
      (unless (equal? generated interpreted)
        (error 'm4-canary "generated Racket e2e bytes diverged from interpreter"))

      (renamed-generated-canary)

      (displayln (jsexpr->string interpreted))
      (displayln (format "m4 canary: source deletion/reload, v3 identity/tamper, query/intent/claim/require sequence, generated-Racket byte parity (~a -> ~a)"
                         BASE NEXT)))
    (lambda () (delete-directory/files directory))))

(module+ main
  (match (vector->list (current-command-line-arguments))
    [(list "--canary") (focused-canary)]
    [_ (error 'm4-canary "use --canary")]))
