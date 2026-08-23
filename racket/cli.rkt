#lang racket

(require json
         racket/cmdline
         racket/file
         racket/list
         racket/path
         racket/string
         "frontend.rkt"
         "kernel.rkt"
         "intent-operations.rkt"
         (only-in "m4-canary.rkt" emit-racket-e2e))

(unless (string=? (version) "9.3")
  (error 'clause-cli "Racket 9.3 is required; running ~a" (version)))

(define racket-executable (find-system-path 'exec-file))

(define (literal-clause item)
  (Clause (parsed-item-name item)
          (make-immutable-hash
           (for/list ([role (in-list (parsed-item-roles item))])
             (cons (parsed-role-name role)
                   (Literal (parsed-role-text role)))))))

(define (same-path? left right)
  (equal? (path->complete-path (string->path left))
          (path->complete-path (string->path right))))

(define (branch-name relation-name)
  (first (string-split relation-name "/")))

(define (write-revision! path revision)
  (display-to-file (serialize-revision revision) path #:exists 'truncate))

(define (read-revision path)
  (reload-revision (file->string path)))

(define (run-generated-program program)
  (define directory (make-temporary-file "clause-cli-generated~a" 'directory))
  (define program-path (build-path directory "plan.rkt"))
  (dynamic-wind
    void
    (lambda ()
      (display-to-file program program-path #:exists 'truncate)
      (define output
        (with-output-to-string
          (lambda ()
            (unless (system* racket-executable program-path)
              (error 'clause-cli "generated Racket plan failed")))))
      output)
    (lambda () (delete-directory/files directory))))

(define (run-generated-plan plan)
  (run-generated-program (emit-racket-plan plan)))

(define (parse-paths)
  (define source-arg #f)
  (define revision-arg #f)
  (define positional
    (command-line
     #:program "clause-cli"
     #:once-each
     [("-s" "--source") path "authoring Clause source path"
      (set! source-arg path)]
     [("-r" "--revision") path "persisted revision path"
      (set! revision-arg path)]
     #:args args args))
  (cond
    [(and source-arg revision-arg (null? positional))
     (list source-arg revision-arg)]
    [(and (not source-arg) (not revision-arg) (= (length positional) 2))
     positional]
    [else
     (error 'clause-cli
            "provide exactly one source path and one revision path, either as --source/--revision or positionally")]))

(define (m3-journey elaborated admitted revision-path)
  (define operations (elaboration-operations elaborated))
  (unless (= (length operations) 2)
    (error 'clause-cli "source must contain exactly one claim and one require operation"))
  (define claim-item (findf (lambda (item) (eq? (parsed-item-kind item) 'claim)) operations))
  (define require-item (findf (lambda (item) (eq? (parsed-item-kind item) 'require)) operations))
  (unless (and claim-item require-item)
    (error 'clause-cli "source must contain one claim and one require operation"))
  (define claim-clause (literal-clause claim-item))
  (define require-clause-value (literal-clause require-item))
  (unless (equal? claim-clause require-clause-value)
    (error 'clause-cli "claim and require clauses must be identical for this journey"))
  (write-revision! revision-path admitted)
  (define reloaded-base (read-revision revision-path))
  (define unsatisfied (require-clause reloaded-base require-clause-value))

  (define-values (claimed-branch claim-output)
    (claim (Branch (branch-name (parsed-item-name claim-item)) reloaded-base)
           claim-clause))
  (define claimed (Branch-head claimed-branch))
  (write-revision! revision-path claimed)
  (define reloaded-claimed (read-revision revision-path))
  (define satisfied (require-clause reloaded-claimed require-clause-value))
  (define plan (check-query reloaded-claimed))
  (define interpreted (interpret-plan plan))
  (define interpreted-json (jsexpr->string (query-output->jsexpr interpreted)))
  (define generated-output (run-generated-plan plan))
  (unless (string=? generated-output (string-append interpreted-json "\n"))
    (error 'clause-cli "generated Racket output diverged from interpreter"))

  (displayln (jsexpr->string (require-output->jsexpr unsatisfied)))
  (displayln (jsexpr->string (claim-output->jsexpr claim-output)))
  (displayln (jsexpr->string (require-output->jsexpr satisfied)))
  (displayln interpreted-json)
  (displayln (format "clause cli: persisted/reloaded query, claim, require, generated-Racket parity (~a -> ~a)"
                     (Revision-identity reloaded-base)
                     (Revision-identity reloaded-claimed))))

(define (m4-journey elaborated admitted revision-path)
  (unless (null? (elaboration-operations elaborated))
    (error 'clause-cli "an intent journey cannot contain trailing claim or require blocks"))
  (write-revision! revision-path admitted)
  (define reloaded-base (read-revision revision-path))
  (define base-model (Revision-model reloaded-base))
  (define intents (Model-intents base-model))
  (unless (= (length intents) 1)
    (error 'clause-cli "the author-facing intent journey requires exactly one declared intent"))
  (define selected (first intents))
  (define intent-name (Intent-name selected))
  (define desired (Intent-desired selected))
  (define branch-id (branch-name (Clause-relation desired)))
  (define base-branch (Branch branch-id reloaded-base))

  (define base-query
    (query-output->jsexpr (interpret-plan (check-query reloaded-base))))
  (define base-intent
    (intent-output->jsexpr (intent base-branch intent-name)))
  (define-values (claimed-branch claim-output) (claim base-branch desired))
  (define claimed (Branch-head claimed-branch))
  (write-revision! revision-path claimed)
  (define reloaded-claimed (read-revision revision-path))
  (define reloaded-branch (Branch branch-id reloaded-claimed))
  (define satisfied
    (require-output->jsexpr (require-clause reloaded-claimed desired)))
  (define next-query
    (query-output->jsexpr (interpret-plan (check-query reloaded-claimed))))
  (define next-intent
    (intent-output->jsexpr (intent reloaded-branch intent-name)))
  (define output
    (list "clause-e2e-output-v1"
          base-query
          base-intent
          (claim-output->jsexpr claim-output)
          satisfied
          next-query
          next-intent))
  (define output-bytes (jsexpr->string output))
  (define generated-output
    (run-generated-program
     (emit-racket-e2e (serialize-revision reloaded-base) intent-name)))
  (unless (string=? generated-output (string-append output-bytes "\n"))
    (error 'clause-cli "generated Racket e2e output diverged from interpreter"))
  (displayln output-bytes))

(define (focused-journey source-path revision-path)
  (when (same-path? source-path revision-path)
    (error 'clause-cli "source and revision paths must be different"))
  (define elaborated (elaborate-source (file->string source-path)))
  (define admitted (admit-semantic (elaboration-semantic elaborated)))
  (if (null? (Model-intents (Revision-model admitted)))
      (m3-journey elaborated admitted revision-path)
      (m4-journey elaborated admitted revision-path)))

(define paths (parse-paths))
(focused-journey (first paths) (second paths))
