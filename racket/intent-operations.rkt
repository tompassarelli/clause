#lang typed/racket/base

(require racket/list
         racket/string
         "kernel.rkt")
(require/typed json
  [jsexpr->string (-> Any String)])

(provide (struct-out IntentSelection)
         (struct-out IntentPlan)
         (struct-out IntentOutput)
         intent-select
         intent-propose
         intent-already-satisfied
         intent-unknown
         intent
         intent-output->jsexpr
         emit-racket-intent-operation)

(define-type IntentStatus (U 'proposed 'already-satisfied 'rejected))

;; Selection fixes every input used by the closed evaluator; it never selects a mode.
(struct IntentSelection ([revision : Revision]
                         [intent : Intent]
                         [facts : (Listof Clause)]) #:transparent)
(struct IntentPlan ([id : String] [base : String] [fact : Clause]) #:transparent)
(struct IntentOutput ([status : IntentStatus]
                      [revision : String]
                      [intent : String]
                      [desired : (Option Clause)]
                      [plan : (Option IntentPlan)]
                      [proof : (Option Proof)]
                      [diagnostic : (Option String)]) #:transparent)

(: intent-select (-> Branch String (Option IntentSelection)))
(define (intent-select branch intent-name)
  (define revision (Branch-head branch))
  (define model (Revision-model revision))
  (define selected
    (findf (lambda ([candidate : Intent])
             (string=? intent-name (Intent-name candidate)))
           (Model-intents model)))
  (and selected (IntentSelection revision selected (Model-facts model))))

(: term->jsexpr (-> (U Literal Variable) Any))
(define (term->jsexpr term)
  (if (Literal? term)
      (list "literal" (Literal-text term))
      (list "variable" (Variable-name term))))

(: clause->jsexpr (-> String Clause Any))
(define (clause->jsexpr kind clause)
  (list kind (Clause-relation clause) "roles"
        (for/list : (Listof Any) ([name (in-list (sort (hash-keys (Clause-roles clause)) string<?))])
          (list name (term->jsexpr (hash-ref (Clause-roles clause) name))))))

(: literal-text (-> (U Literal Variable) String))
(define (literal-text term)
  (if (Literal? term)
      (Literal-text term)
      (error 'intent "intent desired clauses must be closed")))

(: clause-role-values (-> Clause (Listof (List String String))))
(define (clause-role-values clause)
  (for/list : (Listof (List String String))
            ([name (in-list (sort (hash-keys (Clause-roles clause)) string<?))])
    (list name (literal-text (hash-ref (Clause-roles clause) name)))))

(: proof-for (-> String Clause Proof))
(define (proof-for revision-id fact)
  (define roles (clause-role-values fact))
  (Proof (string-append "proof/" revision-id "/" (Clause-relation fact) "/"
                        (string-join (map (lambda ([entry : (List String String)])
                                            (format "~a=~a" (first entry) (second entry)))
                                          roles)
                                     ","))
         (Clause-relation fact)
         roles))

(: intent-propose (-> IntentSelection IntentOutput))
(define (intent-propose selection)
  (define revision (IntentSelection-revision selection))
  (define selected (IntentSelection-intent selection))
  (define revision-id (Revision-identity revision))
  (define desired (Intent-desired selected))
  (IntentOutput 'proposed revision-id (Intent-name selected) desired
                (IntentPlan (format "plan/~a/~a" revision-id (Intent-name selected))
                            revision-id
                            desired)
                #f
                #f))

(: intent-already-satisfied (-> IntentSelection Clause IntentOutput))
(define (intent-already-satisfied selection fact)
  (define revision (IntentSelection-revision selection))
  (define selected (IntentSelection-intent selection))
  (define revision-id (Revision-identity revision))
  (IntentOutput 'already-satisfied revision-id (Intent-name selected) (Intent-desired selected)
                #f
                (proof-for revision-id fact)
                #f))

(: intent-unknown (-> Branch String IntentOutput))
(define (intent-unknown branch intent-name)
  (IntentOutput 'rejected (Revision-identity (Branch-head branch)) intent-name #f #f #f
                "intent.unknown"))

(: intent (-> Branch String IntentOutput))
(define (intent branch intent-name)
  (define selected (intent-select branch intent-name))
  (cond
    [(not selected) (intent-unknown branch intent-name)]
    [else
     (define selection (assert selected IntentSelection?))
     (define desired (Intent-desired (IntentSelection-intent selection)))
     (define matching-fact
       (findf (lambda ([fact : Clause]) (equal? fact desired))
              (IntentSelection-facts selection)))
     (if matching-fact
         (intent-already-satisfied selection (assert matching-fact Clause?))
         (intent-propose selection))]))

(: proof->jsexpr (-> Proof Any))
(define (proof->jsexpr proof)
  (list "proof" (Proof-id proof)
        "relation" (Proof-relation proof)
        "roles" (Proof-roles proof)))

(: intent-output->jsexpr (-> IntentOutput Any))
(define (intent-output->jsexpr output)
  (case (IntentOutput-status output)
    [(proposed)
     (define desired (assert (IntentOutput-desired output) Clause?))
     (define plan (assert (IntentOutput-plan output) IntentPlan?))
     (list "clause-intent-output-v1" "proposed"
           (list "revision" (IntentOutput-revision output))
           (list "intent" (IntentOutput-intent output))
           (list "desired" (clause->jsexpr "clause" desired))
           (list "plan" (list "plan" (IntentPlan-id plan)
                              "operation" "claim"
                              "base" (IntentPlan-base plan)
                              "fact" (clause->jsexpr "fact" (IntentPlan-fact plan))))
           (list "explanation"
                 (list "explanation" "desired-clause-is-absent"
                       "revision" (IntentOutput-revision output)
                       "clause" (clause->jsexpr "clause" desired)
                       "diagnostic" "require.unsatisfied")))]
    [(already-satisfied)
     (define desired (assert (IntentOutput-desired output) Clause?))
     (define proof (assert (IntentOutput-proof output) Proof?))
     (list "clause-intent-output-v1" "already-satisfied"
           (list "revision" (IntentOutput-revision output))
           (list "intent" (IntentOutput-intent output))
           (list "desired" (clause->jsexpr "clause" desired))
           (list "proof" (proof->jsexpr proof))
           (list "explanation"
                 (list "explanation" "desired-clause-is-claimed"
                       "revision" (IntentOutput-revision output))))]
    [(rejected)
     (list "clause-intent-output-v1" "rejected"
           (list "revision" (IntentOutput-revision output))
           (list "intent" (IntentOutput-intent output))
           (list "diagnostic" (assert (IntentOutput-diagnostic output) string?)))]))

(: closed-clause-datum (-> Clause Any))
(define (closed-clause-datum clause)
  (list (Clause-relation clause) (clause-role-values clause)))

(: emit-racket-intent-operation (-> IntentSelection String))
(define (emit-racket-intent-operation selection)
  (define revision (IntentSelection-revision selection))
  (define selected (IntentSelection-intent selection))
  (define lowered
    (list (Revision-identity revision)
          (Intent-name selected)
          (closed-clause-datum (Intent-desired selected))
          (map closed-clause-datum (IntentSelection-facts selection))))
  ;; The generated program rechecks closed fact membership instead of embedding an outcome.
  (string-append
   "#lang racket/base\n(require json racket/list racket/string)\n"
   "(define intent '" (format "~s" lowered) ")\n"
   "(define revision-id (list-ref intent 0))\n"
   "(define intent-name (list-ref intent 1))\n"
   "(define desired (list-ref intent 2))\n"
   "(define facts (list-ref intent 3))\n"
   "(define relation (first desired))\n"
   "(define roles (second desired))\n"
   "(define (clause kind)\n"
   "  (list kind relation \"roles\"\n"
   "        (for/list ([entry (in-list roles)])\n"
   "          (list (first entry) (list \"literal\" (second entry))))))\n"
   "(define (proof)\n"
   "  (list \"proof\"\n"
   "        (string-append \"proof/\" revision-id \"/\" relation \"/\"\n"
   "                       (string-join\n"
   "                        (map (lambda (entry) (format \"~a=~a\" (first entry) (second entry))) roles)\n"
   "                        \",\"))\n"
   "        \"relation\" relation \"roles\" roles))\n"
   "(define output\n"
   "  (if (member desired facts)\n"
   "      (list \"clause-intent-output-v1\" \"already-satisfied\"\n"
   "            (list \"revision\" revision-id)\n"
   "            (list \"intent\" intent-name)\n"
   "            (list \"desired\" (clause \"clause\"))\n"
   "            (list \"proof\" (proof))\n"
   "            (list \"explanation\"\n"
   "                  (list \"explanation\" \"desired-clause-is-claimed\"\n"
   "                        \"revision\" revision-id)))\n"
   "      (list \"clause-intent-output-v1\" \"proposed\"\n"
   "            (list \"revision\" revision-id)\n"
   "            (list \"intent\" intent-name)\n"
   "            (list \"desired\" (clause \"clause\"))\n"
   "            (list \"plan\"\n"
   "                  (list \"plan\" (string-append \"plan/\" revision-id \"/\" intent-name)\n"
   "                        \"operation\" \"claim\" \"base\" revision-id\n"
   "                        \"fact\" (clause \"fact\")))\n"
   "            (list \"explanation\"\n"
   "                  (list \"explanation\" \"desired-clause-is-absent\"\n"
   "                        \"revision\" revision-id\n"
   "                        \"clause\" (clause \"clause\")\n"
   "                        \"diagnostic\" \"require.unsatisfied\"))))\n"
   "(displayln (jsexpr->string output))\n"))
