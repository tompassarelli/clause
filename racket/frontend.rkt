#lang racket

(require racket/list
         racket/match
         racket/string)

(provide (struct-out source-span)
         (struct-out parsed-role)
         (struct-out parsed-item)
         (struct-out elaboration)
         parse-source
         elaborate-source)

;; Spans preserve authored role values without admitting source text as identity.
(struct source-span (start end line column width) #:transparent)
(struct parsed-role (name text span) #:transparent)
(struct parsed-item (kind name roles span sentence mode) #:transparent)
(struct elaboration (semantic value-spans operations) #:transparent)

(define name-rx #px"^[A-Za-z][A-Za-z0-9_/-]*$")
(define role-name-rx #px"^[A-Za-z][A-Za-z0-9_-]*$")
(define value-pattern "[A-Za-z][A-Za-z0-9_/?-]*")
(define quoted-pattern "\"[^\"]+\"")
(define token-pattern (string-append "(?:" quoted-pattern "|\\??" value-pattern ")"))
(define value-rx (pregexp (string-append "^\\??" value-pattern "$")))
(define quoted-rx (pregexp (string-append "^" quoted-pattern "$")))

(define (fail line column message . args)
  (error 'clause-source "line ~a, column ~a: ~a"
         line column (apply format message args)))

(define (line-records source)
  (define offset 0)
  (for/list ([text (in-list (string-split source "\n" #:trim? #f))]
             [number (in-naturals 1)]
             #:do [(define start offset)
                   (set! offset (+ offset (string-length text) 1))]
             #:unless (string=? (string-trim text) ""))
    (list text number start)))

(define (line-text line) (first line))
(define (line-number line) (second line))
(define (line-offset line) (third line))

(define (parse-declared-roles text line base-column base-offset)
  (when (string=? (string-trim text) "")
    (fail line base-column "role list cannot be empty"))
  (for/list ([raw (in-list (string-split text "," #:trim? #f))]
             [part-offset (in-list
                           (let loop ([parts (string-split text "," #:trim? #f)] [at 0])
                             (if (null? parts) '()
                                 (cons at (loop (cdr parts) (+ at (string-length (car parts)) 1))))))])
    (define item (string-trim raw))
    (match (regexp-match #px"^([A-Za-z][A-Za-z0-9_-]*): (Text)$" item)
      [(list _ name type)
       (parsed-role name type
                    (source-span base-offset (+ base-offset (string-length type))
                                 line base-column (string-length type)))]
      [_ (fail line (+ base-column part-offset) "malformed named typed role")])) )

(define (role-map roles who)
  (define names (map parsed-role-name roles))
  (unless (= (length names) (length (remove-duplicates names)))
    (error 'clause-elaborate "~a repeats a named role" who))
  (make-immutable-hash (map (lambda (role) (cons (parsed-role-name role) role)) roles)))

(define (value-role name text line column offset)
  (unless (or (regexp-match? value-rx text)
              (regexp-match? quoted-rx text)
              (and (string-prefix? text "?")
                   (regexp-match? value-rx (substring text 1))))
    (fail line column "malformed role value"))
  (define semantic-text
    (if (regexp-match? quoted-rx text)
        (substring text 1 (sub1 (string-length text)))
        text))
  (parsed-role name semantic-text
               (source-span offset (+ offset (string-length text)) line column (string-length text))))

(define (parse-closed-clause line relation-name sentence-left sentence-literal sentence-right
                             declared who)
  (define clause-rx
    (pregexp (format "^    (~a) ~a (~a)$" token-pattern sentence-literal token-pattern)))
  (define matched (regexp-match clause-rx (line-text line)))
  (unless matched
    (fail (line-number line) 1 "expected ~a matching declared sentence" who))
  (define left (second matched))
  (define right (third matched))
  (when (or (string-prefix? left "?") (string-prefix? right "?"))
    (fail (line-number line) 5 "~a clauses cannot contain open values" who))
  (define left-column 5)
  (define right-column (+ 6 (string-length left) (string-length sentence-literal)))
  (parsed-item who relation-name
               (list (value-role sentence-left left (line-number line) left-column
                                 (+ (line-offset line) 4))
                     (value-role sentence-right right (line-number line) right-column
                                 (+ (line-offset line) (sub1 right-column))))
               (source-span (line-offset line) (+ (line-offset line) (string-length (line-text line)))
                            (line-number line) 1 (string-length (line-text line)))
               (list sentence-left sentence-literal sentence-right) #f))

(define (parse-source source)
  (define lines (line-records source))
  (when (< (length lines) 7) (error 'clause-source "source is incomplete"))
  (define relation-line (first lines))
  (define relation-match
    (regexp-match #px"^relation ([A-Za-z][A-Za-z0-9_/-]*)\\((.*)\\):$" (line-text relation-line)))
  (unless relation-match
    (fail (line-number relation-line) 1 "expected relation declaration"))
  (define relation-name (second relation-match))
  (define raw-roles (third relation-match))
  (define roles-start (+ (line-offset relation-line) (string-length "relation ")
                         (string-length relation-name) 1))
  (define roles-column (+ (string-length "relation ") (string-length relation-name) 2))
  (define declared-roles
    (parse-declared-roles raw-roles (line-number relation-line) roles-column roles-start))
  (define declared (role-map declared-roles "relation declaration"))
  (define sentence-line (second lines))
  (define sentence-match
    (regexp-match #px"^    sentence: \\{([A-Za-z][A-Za-z0-9_-]*)\\} ([A-Za-z][A-Za-z0-9_-]*) \\{([A-Za-z][A-Za-z0-9_-]*)\\}$"
                  (line-text sentence-line)))
  (unless sentence-match
    (fail (line-number sentence-line) 1 "expected one exact mixfix sentence"))
  (define sentence-left (second sentence-match))
  (define sentence-literal (third sentence-match))
  (define sentence-right (fourth sentence-match))
  (unless (and (hash-has-key? declared sentence-left)
               (hash-has-key? declared sentence-right)
               (not (string=? sentence-left sentence-right)))
    (fail (line-number sentence-line) 1 "sentence roles must be distinct declared roles"))
  (define mode-line (third lines))
  (define mode-match
    (regexp-match #px"^    mode ([A-Za-z][A-Za-z0-9_-]*) -> ([A-Za-z][A-Za-z0-9_-]*): many$"
                  (line-text mode-line)))
  (unless mode-match
    (fail (line-number mode-line) 1 "expected finite mode with many cardinality"))
  (define known-role (second mode-match))
  (define sought-role (third mode-match))
  (unless (and (hash-has-key? declared known-role)
               (hash-has-key? declared sought-role)
               (not (string=? known-role sought-role)))
    (fail (line-number mode-line) 1 "mode roles must be distinct declared roles"))
  (define model-line (fourth lines))
  (define model-match (regexp-match #px"^model ([A-Za-z][A-Za-z0-9_-]*):$" (line-text model-line)))
  (unless model-match (fail (line-number model-line) 1 "expected model block"))
  (define model-name (second model-match))
  (unless (string-prefix? relation-name (string-append model-name "/"))
    (fail (line-number model-line) 1 "model does not name the relation namespace"))
  (define query-index
    (for/first ([line (in-list lines)] [index (in-naturals)]
                #:when (regexp-match? #px"^query " (line-text line))) index))
  (unless query-index (error 'clause-source "expected query block"))
  (define clause-rx
    (pregexp (format "^    (~a) ~a (~a)$" token-pattern sentence-literal token-pattern)))
  (define (parse-clause line kind name error-message closed-error)
    (define matched (regexp-match clause-rx (line-text line)))
    (unless matched (fail (line-number line) 1 error-message))
    (define left (second matched))
    (define right (third matched))
    (when (or (string-prefix? left "?") (string-prefix? right "?"))
      (fail (line-number line) 5 closed-error))
    (define left-column 5)
    (define right-column (+ 6 (string-length left) (string-length sentence-literal)))
    (parsed-item kind name
                 (list (value-role sentence-left left (line-number line) left-column
                                   (+ (line-offset line) 4))
                       (value-role sentence-right right (line-number line) right-column
                                   (+ (line-offset line) (sub1 right-column))))
                 (source-span (line-offset line) (+ (line-offset line) (string-length (line-text line)))
                              (line-number line) 1 (string-length (line-text line)))
                 (list sentence-left sentence-literal sentence-right) #f))
  (define facts '())
  (define intents '())
  (define intent-names '())
  (define intent-line-rx #px"^intent ([A-Za-z][A-Za-z0-9_-]*/[A-Za-z][A-Za-z0-9_/-]*):$")
  (let loop ([index 4] [reading-intents? #f])
    (unless (= index query-index)
      (define line (list-ref lines index))
      (define intent-match (regexp-match intent-line-rx (line-text line)))
      (cond
        [intent-match
         (define intent-name (second intent-match))
         (when (member intent-name intent-names)
           (fail (line-number line) 1 "duplicate intent name ~a" intent-name))
         (unless (string-prefix? intent-name (string-append model-name "/"))
           (fail (line-number line) 1 "intent name must begin with model namespace ~a/" model-name))
         (define clause-index (add1 index))
         (when (>= clause-index query-index)
           (fail (line-number line) 1 "intent requires exactly one closed clause"))
         (define intent-clause-line (list-ref lines clause-index))
         (define intent
           (parse-clause intent-clause-line 'intent intent-name
                        "expected one closed intent clause matching declared sentence"
                        "intent clauses cannot contain open values"))
         (set! intent-names (cons intent-name intent-names))
         (set! intents (cons intent intents))
         (loop (add1 clause-index) #t)]
        [reading-intents?
         (fail (line-number line) 1 "expected intent declaration")]
        [else
         (set! facts
               (cons (parse-clause line 'fact relation-name
                                   "expected bare clause matching declared sentence"
                                   "model clauses cannot contain open values")
                     facts))
         (loop (add1 index) #f)])))
  (set! facts (reverse facts))
  (set! intents (reverse intents))
  (unless (pair? facts) (error 'clause-source "model requires a bare clause"))
  (define query-line (list-ref lines query-index))
  (define query-match (regexp-match #px"^query ([A-Za-z][A-Za-z0-9_-]*):$" (line-text query-line)))
  (unless query-match (fail (line-number query-line) 1 "expected query block"))
  (unless (string=? model-name (second query-match))
    (fail (line-number query-line) 1 "query must name the model"))
  (unless (< query-index (sub1 (length lines)))
    (fail (line-number query-line) 1 "query has exactly one clause"))
  (define query-clause-line (list-ref lines (add1 query-index)))
  (define query-rx
    (pregexp (format "^    \\?([A-Za-z][A-Za-z0-9_-]*) where (~a) ~a \\?([A-Za-z][A-Za-z0-9_-]*)$"
                     token-pattern sentence-literal)))
  (define query-clause-match (regexp-match query-rx (line-text query-clause-line)))
  (unless query-clause-match
    (fail (line-number query-clause-line) 1 "expected query matching declared sentence"))
  (define asked-role (second query-clause-match))
  (define known-value (third query-clause-match))
  (define open-role (fourth query-clause-match))
  (unless (and (string=? asked-role open-role) (string=? open-role sought-role)
               (string=? sentence-left known-role) (string=? sentence-right sought-role))
    (fail (line-number query-clause-line) 5 "query must open the declared sought role"))
  (define known-column 19)
  (define open-column (+ known-column (string-length known-value)
                         (string-length sentence-literal) 2))
  (define query
    (parsed-item 'query relation-name
                 (list (value-role known-role known-value (line-number query-clause-line) known-column
                                   (+ (line-offset query-clause-line) (sub1 known-column)))
                       (value-role sought-role (string-append "?" open-role)
                                   (line-number query-clause-line) open-column
                                   (+ (line-offset query-clause-line) (sub1 open-column))))
                 (source-span (line-offset query-clause-line)
                              (+ (line-offset query-clause-line) (string-length (line-text query-clause-line)))
                              (line-number query-clause-line) 1 (string-length (line-text query-clause-line)))
                 (list sentence-left sentence-literal sentence-right) #f))
  (define operation-lines (drop lines (+ query-index 2)))
  (define operations
    (let loop ([remaining operation-lines] [result '()])
      (cond
        [(null? remaining) (reverse result)]
        [(null? (cdr remaining))
         (fail (line-number (car remaining)) 1 "operation requires one clause")]
        [else
         (define header (car remaining))
         (define body (cadr remaining))
         (define operation-match
           (regexp-match #px"^(claim|require) ([A-Za-z][A-Za-z0-9_-]*):$"
                         (line-text header)))
         (unless operation-match
           (fail (line-number header) 1 "expected claim or require block"))
         (define kind (string->symbol (second operation-match)))
         (define operation-name (third operation-match))
         (unless (string=? operation-name model-name)
           (fail (line-number header) 1 "operation must name the model"))
         (loop (cddr remaining)
               (cons (parse-closed-clause body relation-name sentence-left sentence-literal
                                          sentence-right declared kind)
                     result))])))
  (define relation
    (parsed-item 'relation relation-name declared-roles (source-span (line-offset relation-line)
                                                                       (+ (line-offset relation-line)
                                                                          (string-length (line-text relation-line)))
                                                                       (line-number relation-line) 1
                                                                       (string-length (line-text relation-line)))
                 (list sentence-left sentence-literal sentence-right)
                 (list known-role sought-role "many")))
  (append (list relation facts intents query) operations))

(define (elaborate-source source)
  (define parsed (parse-source source))
  (define declaration (first parsed))
  (define facts (second parsed))
  (define intents (third parsed))
  (define query (fourth parsed))
  (define operations (drop parsed 4))
  (define ordered-roles (sort (parsed-item-roles declaration) string<? #:key parsed-role-name))
  (define (clause-semantic clause kind [name (parsed-item-name clause)])
    (list kind name "roles"
          (for/list ([role-name (in-list (map parsed-role-name ordered-roles))])
            (define item (findf (lambda (role) (string=? (parsed-role-name role) role-name))
                                (parsed-item-roles clause)))
            (list role-name
                  (if (string-prefix? (parsed-role-text item) "?")
                      (list "variable" (substring (parsed-role-text item) 1))
                      (list "literal" (parsed-role-text item)))))))
  (define sentence (parsed-item-sentence declaration))
  (define declared-mode (parsed-item-mode declaration))
  (define intent-semantics
    (for/list ([intent (in-list (sort intents string<? #:key parsed-item-name))])
      (list "intent" (parsed-item-name intent) "desired"
            (clause-semantic intent "clause" (parsed-item-name declaration)))))
  (define semantic
    (list "clause-semantic-v3"
          (list "relations"
                (list (list "relation" (parsed-item-name declaration) "roles"
                            (for/list ([role (in-list ordered-roles)])
                              (list (parsed-role-name role) (parsed-role-text role)))
                            "sentence" sentence
                            "modes" (list (list "mode" "finite" "known" (list (first declared-mode))
                                                "sought" (list (second declared-mode))
                                                "cardinality" (third declared-mode))))))
          (list "facts" (sort (map (lambda (fact) (clause-semantic fact "fact")) facts) string<? #:key ~s))
          (list "query" (clause-semantic query "query"))
          (list "intents" intent-semantics)
          (list "order" "ascending" (second declared-mode))))
  (elaboration semantic
               (map parsed-role-span
                    (append (append-map parsed-item-roles facts)
                            (append-map parsed-item-roles intents)
                            (parsed-item-roles query)))
               operations))
