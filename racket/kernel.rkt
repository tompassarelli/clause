#lang typed/racket/base

(require racket/list
         racket/match
         racket/string)
(require/typed json
  [jsexpr->string (-> Any String)]
  [string->jsexpr (-> String Any)])

(provide (struct-out Role)
         (struct-out Literal)
         (struct-out Variable)
         (struct-out Sentence)
         (struct-out Relation)
         (struct-out Clause)
         (struct-out Intent)
         (struct-out Model)
         (struct-out Revision)
         (struct-out Branch)
         (struct-out mode)
         (struct-out CheckedPlan)
         (struct-out Proof)
         (struct-out QueryOutput)
         (struct-out ClaimOutput)
         (struct-out RequireOutput)
         admit-semantic
         serialize-revision
         reload-revision
         claim
         require-clause
         claim-output->jsexpr
         require-output->jsexpr
         check-query
         interpret-plan
         emit-racket-plan
         proof-valid?
         query-output->jsexpr)

(struct Role ([name : String] [type : String]) #:transparent)
(struct Literal ([text : String]) #:transparent)
(struct Variable ([name : String]) #:transparent)
(define-type Term (U Literal Variable))
(struct Sentence ([roles : (List String String)] [literal : String]) #:transparent)
(struct mode ([name : 'finite]
              [known : (Listof String)]
              [sought : (Listof String)]
              [cardinality : 'many]) #:transparent)
(struct Relation ([name : String]
                  [roles : (Immutable-HashTable String Role)]
                  [sentence : Sentence]
                  [modes : (Listof mode)]) #:transparent)
(struct Clause ([relation : String] [roles : (Immutable-HashTable String Term)]) #:transparent)
(struct Intent ([name : String] [desired : Clause]) #:transparent)
(struct Model ([relations : (Immutable-HashTable String Relation)]
               [facts : (Listof Clause)]
               [query : Clause]
               [intents : (Listof Intent)]) #:transparent)
(struct Revision ([identity : String] [model : Model]) #:transparent)
(struct Branch ([name : String] [head : Revision]) #:transparent)
(struct CheckedPlan ([revision-id : String]
                     [mode : mode]
                     [relation : String]
                     [variable-role : String]
                     [requested : (Immutable-HashTable String Term)]
                     [facts : (Listof Clause)]
                     [order : 'ascending]) #:transparent)
(struct Proof ([id : String] [relation : String] [roles : (Listof (List String String))]) #:transparent)
(struct QueryOutput ([results : (Listof String)] [proofs : (Listof Proof)]) #:transparent)
(struct ClaimOutput ([status : (U 'admitted 'duplicate)]
                     [branch : String]
                     [base : (Option String)]
                     [revision : String]
                     [fact : (Option Clause)]
                     [diagnostic : (Option String)]) #:transparent)
(struct RequireOutput ([status : (U 'satisfied 'unsatisfied)]
                       [revision : String]
                       [proof : (Option Proof)]
                       [clause : (Option Clause)]
                       [diagnostic : (Option String)]) #:transparent)

(define name-rx #px"^[A-Za-z][A-Za-z0-9_/-]*$")
(define role-name-rx #px"^[A-Za-z][A-Za-z0-9_-]*$")
(define value-rx #px"^[A-Za-z][A-Za-z0-9_/?-]*$")

(: reject (-> String Nothing))
(define (reject message) (error 'clause-reload "~a" message))

(: expect-string (-> Any String String))
(define (expect-string value where)
  (if (string? value)
      (string->immutable-string value)
      (reject (format "~a must be a string" where))))

(: valid-name (-> Any Regexp String String))
(define (valid-name value rx where)
  (define text (expect-string value where))
  (unless (regexp-match? rx text) (reject (format "invalid ~a" where)))
  text)

(: exact-list (-> Any Natural String (Listof Any)))
(define (exact-list value count where)
  (unless (and (list? value) (= (length value) count))
    (reject (format "invalid ~a" where)))
  (assert value list?))

(: bytes->hex (-> Bytes String))
(define (bytes->hex data)
  (apply string-append
         (for/list : (Listof String) ([byte (in-bytes data)])
           (define digits (number->string byte 16))
           (if (= (string-length digits) 1) (string-append "0" digits) digits))))

(: canonical-model (-> Model Any))
(define (canonical-model model)
  (define relation-list
    (for/list : (Listof Any) ([name (in-list (sort (hash-keys (Model-relations model)) string<?))])
      (define rel (hash-ref (Model-relations model) name))
      (list "relation" name "roles"
            (for/list : (Listof Any) ([role-name (in-list (sort (hash-keys (Relation-roles rel)) string<?))])
              (define item (hash-ref (Relation-roles rel) role-name))
              (list (Role-name item) (Role-type item)))
            "sentence" (list (first (Sentence-roles (Relation-sentence rel)))
                             (Sentence-literal (Relation-sentence rel))
                             (second (Sentence-roles (Relation-sentence rel))))
            "modes"
            (for/list : (Listof Any) ([declared (in-list (Relation-modes rel))])
              (list "mode" "finite" "known" (mode-known declared)
                    "sought" (mode-sought declared) "cardinality" "many")))))
  (: term-datum (-> Term Any))
  (define (term-datum item)
    (if (Literal? item) (list "literal" (Literal-text item))
        (list "variable" (Variable-name item))))
  (: clause-datum (-> String Clause Any))
  (define (clause-datum kind item)
    (list kind (Clause-relation item) "roles"
          (for/list : (Listof Any) ([name (in-list (sort (hash-keys (Clause-roles item)) string<?))])
            (list name (term-datum (hash-ref (Clause-roles item) name))))))
  (define facts
    (sort (map (lambda ([fact : Clause]) (clause-datum "fact" fact))
               (Model-facts model))
          (lambda ([left : Any] [right : Any])
            (string<? (jsexpr->string left) (jsexpr->string right)))))
  (define intents
    (for/list : (Listof Any) ([intent (in-list (sort (Model-intents model)
                                                    (lambda ([left : Intent] [right : Intent])
                                                      (string<? (Intent-name left) (Intent-name right)))))])
      (list "intent" (Intent-name intent) "desired" (clause-datum "clause" (Intent-desired intent)))))
  (define query (Model-query model))
  (define variable-maybe : (Option String)
    (findf (lambda ([name : String]) (Variable? (hash-ref (Clause-roles query) name)))
           (sort (hash-keys (Clause-roles query)) string<?)))
  (define variable (assert variable-maybe string?))
  (list "clause-semantic-v3"
        (list "relations" relation-list)
        (list "facts" facts)
        (list "query" (clause-datum "query" query))
        (list "intents" intents)
        (list "order" "ascending" variable)))

(: identity-for (-> Model String))
(define (identity-for model)
  (string->immutable-string
   (string-append "rev-sha256-"
                  (bytes->hex (sha256-bytes (open-input-bytes
                                             (string->bytes/utf-8
                                              (jsexpr->string (canonical-model model)))))))))

(: decode-term (-> Any String Term))
(define (decode-term value where)
  (match (exact-list value 2 where)
    [(list (and kind (? string?)) raw)
     (define text (valid-name raw value-rx where))
     (cond [(string=? kind "literal") (Literal text)]
           [(string=? kind "variable") (Variable text)]
           [else (reject (format "invalid ~a term kind" where))])]
    [_ (reject (format "invalid ~a term" where))]))

(: decode-model (-> Any Model))
(define (decode-model semantic)
  (match (exact-list semantic 6 "semantic payload")
    [(list "clause-semantic-v3"
           (list "relations" relation-values)
           (list "facts" fact-values)
           (list "query" query-value)
           (list "intents" intent-values)
           (list "order" "ascending" raw-order-role))
     (unless (and (list? relation-values) (= (length relation-values) 1))
       (reject "exactly one relation is required"))
     (define relation-entry (first relation-values))
     (define-values (relation-name role-map role-names sentence modes)
       (match (exact-list relation-entry 8 "relation")
         [(list "relation" raw-name "roles" raw-roles "sentence" raw-sentence "modes" raw-modes)
          (define name (valid-name raw-name name-rx "relation name"))
          (unless (and (list? raw-roles) (pair? raw-roles)) (reject "relation roles must be nonempty"))
          (define names : (Listof String) '())
          (define pairs
            (for/list : (Listof (Pairof String Role)) ([entry (in-list raw-roles)] [index (in-naturals)])
              (match (exact-list entry 2 (format "relation role ~a" index))
                [(list raw-role raw-type)
                 (define role-name (valid-name raw-role role-name-rx "role name"))
                 (when (member role-name names) (reject "duplicate relation role"))
                 (set! names (cons role-name names))
                 (define type-name (expect-string raw-type "role type"))
                 (unless (string=? type-name "Text") (reject "only Text roles are admitted"))
                 (cons role-name (Role role-name type-name))]
                [_ (reject "invalid relation role")])) )
          (define pair-names
            (map (lambda ([entry : (Pairof String Role)]) (car entry)) pairs))
          (unless (equal? pair-names (sort pair-names string<?))
            (reject "relation roles are not canonical"))
          (define sentence-values (exact-list raw-sentence 3 "sentence"))
          (define left (valid-name (first sentence-values) role-name-rx "sentence role"))
          (define literal (valid-name (second sentence-values) role-name-rx "sentence literal"))
          (define right (valid-name (third sentence-values) role-name-rx "sentence role"))
          (unless (and (member left names) (member right names) (not (string=? left right)))
            (reject "sentence roles are invalid"))
          (unless (and (list? raw-modes) (= (length raw-modes) 1))
            (reject "exactly one finite mode is required"))
          (define mode-entry (first (assert raw-modes list?)))
          (define declared-mode
            (match (exact-list mode-entry 8 "mode")
              [(list "mode" "finite" "known" raw-known "sought" raw-sought "cardinality" "many")
               (unless (and (list? raw-known) (= (length raw-known) 1)
                            (list? raw-sought) (= (length raw-sought) 1))
                 (reject "mode must declare one known and one sought role"))
               (define known (map (lambda ([role : Any])
                                    (valid-name role role-name-rx "known role"))
                                  (assert raw-known list?)))
               (define sought (map (lambda ([role : Any])
                                    (valid-name role role-name-rx "sought role"))
                                  (assert raw-sought list?)))
               (unless (and (andmap (lambda ([role : String]) (member role names)) known)
                            (andmap (lambda ([role : String]) (member role names)) sought)
                            (not (member (first known) sought)))
                 (reject "mode roles are invalid"))
               (mode 'finite known sought 'many)]
              [_ (reject "invalid mode")]))
          (values name (make-immutable-hash pairs) (sort names string<?)
                  (Sentence (list left right) literal) (list declared-mode))]
         [_ (reject "invalid relation")]))
     (: decode-clause (-> Any String Boolean Clause))
     (define (decode-clause value expected-kind closed?)
       (match (exact-list value 4 expected-kind)
         [(list kind raw-relation "roles" raw-roles)
          (unless (equal? kind expected-kind) (reject (format "invalid ~a kind" expected-kind)))
          (unless (equal? raw-relation relation-name) (reject (format "invalid ~a relation" expected-kind)))
          (unless (list? raw-roles) (reject (format "invalid ~a roles" expected-kind)))
          (define seen : (Listof String) '())
          (define pairs
            (for/list : (Listof (Pairof String Term)) ([entry (in-list raw-roles)] [index (in-naturals)])
              (match (exact-list entry 2 (format "~a role ~a" expected-kind index))
                [(list raw-name raw-term)
                 (define name (valid-name raw-name role-name-rx "binding role"))
                 (when (member name seen) (reject (format "duplicate ~a role" expected-kind)))
                 (set! seen (cons name seen))
                 (define item (decode-term raw-term (format "~a.~a" expected-kind name)))
                 (when (and closed? (Variable? item))
                   (reject (format "~a cannot contain variables" expected-kind)))
                 (cons name item)]
                [_ (reject (format "invalid ~a role" expected-kind))])))
          (define pair-names
            (map (lambda ([entry : (Pairof String Term)]) (car entry)) pairs))
          (unless (equal? pair-names role-names)
            (reject (format "incomplete or unordered ~a role map" expected-kind)))
          (Clause relation-name (make-immutable-hash pairs))]
         [_ (reject (format "invalid ~a" expected-kind))]))
     (unless (and (list? fact-values) (pair? fact-values)) (reject "facts must be nonempty"))
     (define fact-list : (Listof Any) (assert fact-values list?))
     (define facts (map (lambda ([item : Any]) (decode-clause item "fact" #t)) fact-list))
     (define query (decode-clause query-value "query" #f))
     (unless (list? intent-values) (reject "intents must be an array"))
     (define relation-namespace
       (let ([parts (string-split relation-name "/")])
         (unless (>= (length parts) 2) (reject "relation must have a namespace"))
         (string-append (first parts) "/")))
     (define intent-names : (Listof String) '())
     (define intents
       (for/list : (Listof Intent) ([entry (in-list (assert intent-values list?))]
                                     [index (in-naturals)])
         (match (exact-list entry 4 (format "intent ~a" index))
           [(list "intent" raw-name "desired" desired-value)
            (define intent-name (valid-name raw-name name-rx "intent name"))
            (unless (string-prefix? intent-name relation-namespace)
              (reject "intent name is outside the model namespace"))
            (when (member intent-name intent-names) (reject "duplicate intent name"))
            (set! intent-names (cons intent-name intent-names))
            (Intent intent-name (decode-clause desired-value "clause" #t))]
           [_ (reject "invalid intent")])) )
     (define order-role (valid-name raw-order-role role-name-rx "order role"))
     (define variables
       (for/list : (Listof String) ([(name item) (in-hash (Clause-roles query))] #:when (Variable? item)) name))
     (unless (= (length variables) 1) (reject "query must bind exactly one variable"))
     (unless (equal? order-role (first variables)) (reject "query order role is invalid"))
     (define model (Model (hash relation-name (Relation relation-name role-map sentence modes)) facts query intents))
     (unless (equal? semantic (canonical-model model)) (reject "semantic payload is not canonical"))
     model]
    [_ (reject "invalid semantic payload")]))

(: admit-semantic (-> Any Revision))
(define (admit-semantic semantic)
  (define model (decode-model semantic))
  (Revision (identity-for model) model))

(: serialize-revision (-> Revision String))
(define (serialize-revision revision)
  (jsexpr->string (list "clause-revision-v1" (Revision-identity revision)
                        (canonical-model (Revision-model revision)))))

(: reload-revision (-> String Revision))
(define (reload-revision text)
  (define envelope (string->jsexpr text))
  (unless (string=? text (jsexpr->string envelope))
    (reject "revision envelope is not canonical"))
  (match (exact-list envelope 3 "revision envelope")
    [(list "clause-revision-v1" raw-id semantic)
     (define stored-id (expect-string raw-id "revision identity"))
     (unless (regexp-match? #px"^rev-sha256-[0-9a-f]{64}$" stored-id)
       (reject "invalid revision identity"))
     (define model (decode-model semantic))
     (define expected-id (identity-for model))
     (unless (string=? stored-id expected-id)
       (reject "revision identity does not match canonical semantic payload"))
     (Revision stored-id model)]
    [_ (reject "invalid revision envelope")]))

(: check-query (-> Revision CheckedPlan))
(define (check-query revision)
  (define model (Revision-model revision))
  (define query (Model-query model))
  (define relation-name (Clause-relation query))
  (unless (hash-has-key? (Model-relations model) relation-name) (error 'clause-plan "unknown relation"))
  (define relation (hash-ref (Model-relations model) relation-name))
  (define variable-roles
    (sort (for/list : (Listof String) ([(name item) (in-hash (Clause-roles query))]
                                       #:when (Variable? item)) name)
          string<?))
  (define known-roles
    (sort (for/list : (Listof String) ([(name item) (in-hash (Clause-roles query))]
                                       #:when (Literal? item)) name)
          string<?))
  (define selected-mode-maybe : (Option mode)
    (findf (lambda ([candidate : mode])
             (and (equal? (mode-known candidate) known-roles)
                  (equal? (mode-sought candidate) variable-roles)
                  (eq? (mode-cardinality candidate) 'many)))
           (Relation-modes relation)))
  (define selected-mode (assert selected-mode-maybe mode?))
  (define variable-role (first variable-roles))
  (CheckedPlan (Revision-identity revision) selected-mode relation-name variable-role
               (Clause-roles query) (Model-facts model) 'ascending))

(: literal-text (-> Term String))
(define (literal-text item)
  (if (Literal? item) (Literal-text item) (error 'clause-query "fact contains variable")))

(: clause-role-values (-> Clause (Listof (List String String))))
(define (clause-role-values fact)
  (for/list : (Listof (List String String)) ([name (in-list (sort (hash-keys (Clause-roles fact)) string<?))])
    (list name (literal-text (hash-ref (Clause-roles fact) name)))))

(: proof-for (-> String Clause Proof))
(define (proof-for revision-id fact)
  (define roles (clause-role-values fact))
  (Proof (string-append "proof/" revision-id "/" (Clause-relation fact) "/"
                        (string-join (map (lambda ([entry : (List String String)])
                                            (format "~a=~a" (first entry) (second entry))) roles) ","))
         (Clause-relation fact) roles))

;; M3 deliberately keeps claim and require closed and mode-free.  They operate
;; on a Branch/Revision's admitted facts; query planning remains the only place
;; that chooses a declared mode.
(: closed-complete-clause? (-> Model Clause Boolean))
(define (closed-complete-clause? model clause)
  (and (hash-has-key? (Model-relations model) (Clause-relation clause))
       (let ([relation (hash-ref (Model-relations model) (Clause-relation clause))])
         (and (equal? (sort (hash-keys (Clause-roles clause)) string<?)
                      (sort (hash-keys (Relation-roles relation)) string<?))
              (for/and : Boolean ([(name term) (in-hash (Clause-roles clause))])
                (and (hash-has-key? (Relation-roles relation) name)
                     (Literal? term)
                     (regexp-match? value-rx (Literal-text (assert term Literal?)))))))))

(: clause->jsexpr (-> String Clause Any))
(define (clause->jsexpr kind clause)
  (: term-datum (-> Term Any))
  (define (term-datum item)
    (if (Literal? item) (list "literal" (Literal-text item))
        (list "variable" (Variable-name item))))
  (list kind (Clause-relation clause) "roles"
        (for/list : (Listof Any) ([name (in-list (sort (hash-keys (Clause-roles clause)) string<?))])
          (list name (term-datum (hash-ref (Clause-roles clause) name))))))

(: canonical-facts (-> (Listof Clause) (Listof Clause)))
(define (canonical-facts facts)
  (sort facts
        (lambda ([left : Clause] [right : Clause])
          (string<? (jsexpr->string (clause->jsexpr "fact" left))
                    (jsexpr->string (clause->jsexpr "fact" right))))))

(: claim-output->jsexpr (-> ClaimOutput Any))
(define (claim-output->jsexpr output)
  (match output
    [(ClaimOutput 'admitted branch (? string? base) revision (? Clause? fact) #f)
     (list "clause-claim-output-v1" "admitted"
           (list "branch" branch)
           (list "base" base)
           (list "revision" revision)
           (list "fact" (clause->jsexpr "fact" fact)))]
    [(ClaimOutput 'duplicate branch #f revision #f (? string? diagnostic))
     (list "clause-claim-output-v1" "duplicate"
           (list "branch" branch)
           (list "revision" revision)
           (list "diagnostic" diagnostic))]
    [_ (error 'clause-claim "invalid claim output")]))

(: require-output->jsexpr (-> RequireOutput Any))
(define (require-output->jsexpr output)
  (match output
    [(RequireOutput 'satisfied revision (? Proof? proof) #f #f)
     (list "clause-require-output-v1" "satisfied"
           (list "revision" revision)
           (list "proof" (list "proof" (Proof-id proof)
                                "relation" (Proof-relation proof)
                                "roles" (Proof-roles proof))))]
    [(RequireOutput 'unsatisfied revision #f (? Clause? clause) (? string? diagnostic))
     (list "clause-require-output-v1" "unsatisfied"
           (list "revision" revision)
           (list "clause" (clause->jsexpr "clause" clause))
           (list "diagnostic" diagnostic))]
    [_ (error 'clause-require "invalid require output")]))

(: claim (-> Branch Clause (Values Branch ClaimOutput)))
(define (claim branch clause)
  (define base (Branch-head branch))
  (define base-model (Revision-model base))
  (unless (closed-complete-clause? base-model clause)
    (error 'clause-claim "claim requires a complete closed clause for a declared relation"))
  (if (member clause (Model-facts base-model))
      (values branch
              (ClaimOutput 'duplicate (Branch-name branch) #f
                           (Revision-identity base) #f "claim.duplicate"))
      (let* ([next-model (Model (Model-relations base-model)
                                (canonical-facts (append (Model-facts base-model) (list clause)))
                                (Model-query base-model)
                                (Model-intents base-model))]
             [next-revision (Revision (identity-for next-model) next-model)]
             [next-branch (Branch (Branch-name branch) next-revision)])
        (values next-branch
                (ClaimOutput 'admitted (Branch-name branch) (Revision-identity base)
                             (Revision-identity next-revision) clause #f)))))

(: require-clause (-> Revision Clause RequireOutput))
(define (require-clause revision clause)
  (define model (Revision-model revision))
  (unless (closed-complete-clause? model clause)
    (error 'clause-require "require needs a complete closed clause for a declared relation"))
  (define matching-fact-maybe : (Option Clause)
    (findf (lambda ([fact : Clause]) (equal? fact clause)) (Model-facts model)))
  (if matching-fact-maybe
      (let ([fact (assert matching-fact-maybe Clause?)])
        (RequireOutput 'satisfied (Revision-identity revision)
                       (proof-for (Revision-identity revision) fact) #f #f))
      (RequireOutput 'unsatisfied (Revision-identity revision) #f clause
                     "require.unsatisfied")))

(: interpret-plan (-> CheckedPlan QueryOutput))
(define (interpret-plan plan)
  (define requested (CheckedPlan-requested plan))
  (define variable (CheckedPlan-variable-role plan))
  (define matches
    (filter
     (lambda ([fact : Clause])
       (and (string=? (Clause-relation fact) (CheckedPlan-relation plan))
            (for/and : Boolean ([(name wanted) (in-hash requested)])
              (or (Variable? wanted)
                  (string=? (Literal-text (assert wanted Literal?))
                            (literal-text (hash-ref (Clause-roles fact) name)))))))
     (CheckedPlan-facts plan)))
  (define ordered
    (sort matches
          (lambda ([left : Clause] [right : Clause])
            (string<? (literal-text (hash-ref (Clause-roles left) variable))
                      (literal-text (hash-ref (Clause-roles right) variable))))))
  (QueryOutput (map (lambda ([fact : Clause]) (literal-text (hash-ref (Clause-roles fact) variable))) ordered)
               (map (lambda ([fact : Clause]) (proof-for (CheckedPlan-revision-id plan) fact)) ordered)))

(: proof-valid? (-> Revision Proof Boolean))
(define (proof-valid? revision proof)
  (for/or : Boolean ([fact (in-list (Model-facts (Revision-model revision)))])
    (equal? proof (proof-for (Revision-identity revision) fact))))

(: query-output->jsexpr (-> QueryOutput Any))
(define (query-output->jsexpr output)
  (list "clause-query-output-v1"
        (list "results" (QueryOutput-results output))
        (list "proofs"
              (for/list : (Listof Any) ([proof (in-list (QueryOutput-proofs output))])
                (list "proof" (Proof-id proof)
                      "relation" (Proof-relation proof)
                      "roles" (Proof-roles proof))))))

(: emit-racket-plan (-> CheckedPlan String))
(define (emit-racket-plan plan)
  (: term-datum (-> Term Any))
  (define (term-datum item)
    (if (Literal? item) (list 'literal (Literal-text item))
        (list 'variable (Variable-name item))))
  (: roles-datum (-> (Immutable-HashTable String Term) Any))
  (define (roles-datum roles)
    (for/list : (Listof Any) ([name (in-list (sort (hash-keys roles) string<?))])
      (list name (term-datum (hash-ref roles name)))))
  (define lowered
    (list (CheckedPlan-revision-id plan)
          (CheckedPlan-relation plan)
          (CheckedPlan-variable-role plan)
          (roles-datum (CheckedPlan-requested plan))
          (for/list : (Listof Any) ([fact (in-list (CheckedPlan-facts plan))])
            (list (Clause-relation fact) (roles-datum (Clause-roles fact))))))
  ;; This is a closed, finite Racket evaluator for the checked plan datum. It
  ;; performs filtering, ordering, projection, and proof construction at run
  ;; time; no interpreter result is embedded in the generated module.
  (string-append
   "#lang racket/base\n(require json racket/list racket/string)\n"
   "(define plan '" (format "~s" lowered) ")\n"
   "(define revision-id (list-ref plan 0))\n"
   "(define relation (list-ref plan 1))\n"
   "(define variable (list-ref plan 2))\n"
   "(define requested (list-ref plan 3))\n"
   "(define facts (list-ref plan 4))\n"
   "(define (role-term roles name) (second (assoc name roles)))\n"
   "(define matches\n"
   "  (filter (lambda (fact)\n"
   "            (and (string=? (first fact) relation)\n"
   "                 (for/and ([wanted (in-list requested)])\n"
   "                   (or (eq? (first (second wanted)) 'variable)\n"
   "                       (string=? (second (second wanted))\n"
   "                                 (second (role-term (second fact) (first wanted))))))))\n"
   "          facts))\n"
   "(define ordered\n"
   "  (sort matches string<? #:key\n"
   "        (lambda (fact) (second (role-term (second fact) variable)))))\n"
   "(define (role-values fact)\n"
   "  (for/list ([entry (in-list (second fact))])\n"
   "    (list (first entry) (second (second entry)))))\n"
   "(define (proof fact)\n"
   "  (define roles (role-values fact))\n"
   "  (list \"proof\" (string-append \"proof/\" revision-id \"/\" (first fact) \"/\"\n"
   "                           (string-join (map (lambda (entry)\n"
   "                                               (format \"~a=~a\" (first entry) (second entry)))\n"
   "                                             roles) \",\"))\n"
   "        \"relation\" (first fact) \"roles\" roles))\n"
   "(define output\n"
   "  (list \"clause-query-output-v1\"\n"
   "        (list \"results\" (map (lambda (fact) (second (role-term (second fact) variable))) ordered))\n"
   "        (list \"proofs\" (map proof ordered))))\n"
   "(displayln (jsexpr->string output))\n"))
