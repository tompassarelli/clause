// Untrusted literal materializer; it cannot authorize any emitted package.
use clause_substrate::compiler_package_v3::*;
use clause_substrate::evaluator::Evaluator;
use std::fs;
use std::path::Path;

const TAG: &[u8] = b"clause/core-abi/tag/v1";
const BYTES: &[u8] = b"clause/core-abi/bytes/v1";
const ID: &[u8] = b"clause/core-abi/id32/v1";
const U64: &[u8] = b"clause/core-abi/u64/v1";
const EQ: &[u8] = b"clause/core/bytes-equal/v1";
fn bx<T>(x: T) -> FallibleBox<T> {
    FallibleBox::try_new(x).unwrap()
}
fn atom(kind: &[u8], payload: &[u8]) -> Term {
    Term::Atom {
        kind: kind.to_vec(),
        canonical_payload: payload.to_vec(),
        equality_contract: EQ.to_vec(),
    }
}
fn bytes(x: &[u8]) -> Term {
    atom(BYTES, x)
}
fn tag(x: u8) -> Term {
    atom(TAG, &[x])
}
fn nil() -> Term {
    tag(0)
}
fn triple(a: Term, b: Term, c: Term) -> Term {
    Term::Triple(bx(a), bx(b), bx(c))
}
fn list(xs: Vec<Term>) -> Term {
    xs.into_iter()
        .rev()
        .fold(nil(), |tail, x| triple(tag(1), x, tail))
}
fn record(t: u8, xs: Vec<Term>) -> Term {
    triple(tag(t), list(xs), nil())
}
fn ident(x: u32) -> Id32 {
    let mut b = [0; 32];
    b[0] = 0x31;
    b[28..].copy_from_slice(&x.to_be_bytes());
    Id32(b)
}
fn idterm(x: Id32) -> Term {
    atom(ID, x.as_bytes())
}
fn number(x: u64) -> Term {
    atom(U64, &x.to_be_bytes())
}
fn clone_term(x: &Term) -> Term {
    match x {
        Term::Atom {
            kind,
            canonical_payload,
            equality_contract,
        } => Term::Atom {
            kind: kind.clone(),
            canonical_payload: canonical_payload.clone(),
            equality_contract: equality_contract.clone(),
        },
        Term::Triple(a, b, c) => triple(clone_term(a), clone_term(b), clone_term(c)),
    }
}
#[derive(Clone)]
enum E {
    B(Vec<u8>),
    T(std::sync::Arc<Term>),
    V(&'static str),
    Atom(Box<E>, Box<E>, Box<E>),
    Triple(Box<E>, Box<E>, Box<E>),
    Let(&'static str, Box<E>, Box<E>),
    Case(Box<E>, [&'static str; 3], Box<E>, Box<E>),
    Byte(Box<E>, [&'static str; 2], Box<E>, Box<E>),
    Cat(Vec<E>),
    Eq(Box<E>, Box<E>, Box<E>, Box<E>),
    Call(u32, Vec<E>),
    Sha(Box<E>),
}
fn b(x: impl AsRef<[u8]>) -> E {
    E::B(x.as_ref().to_vec())
}
fn v(x: &'static str) -> E {
    E::V(x)
}
fn t(x: Term) -> E {
    E::T(std::sync::Arc::new(x))
}
fn call(id: u32, xs: Vec<E>) -> E {
    E::Call(id, xs)
}
fn cat(xs: Vec<E>) -> E {
    E::Cat(xs)
}
fn eq(a: E, b: E, y: E, n: E) -> E {
    E::Eq(Box::new(a), Box::new(b), Box::new(y), Box::new(n))
}
fn let_(name: &'static str, x: E, body: E) -> E {
    E::Let(name, Box::new(x), Box::new(body))
}
fn ct(x: E, names: [&'static str; 3], a: E, tr: E) -> E {
    E::Case(Box::new(x), names, Box::new(a), Box::new(tr))
}
fn cb(x: E, names: [&'static str; 2], empty: E, cons: E) -> E {
    E::Byte(Box::new(x), names, Box::new(empty), Box::new(cons))
}
fn at(k: E, p: E, q: E) -> E {
    E::Atom(Box::new(k), Box::new(p), Box::new(q))
}
fn tr(a: E, b: E, c: E) -> E {
    E::Triple(Box::new(a), Box::new(b), Box::new(c))
}
fn tb(x: E) -> E {
    at(b(BYTES), x, b(EQ))
}
fn ls(xs: Vec<E>) -> E {
    xs.into_iter()
        .rev()
        .fold(t(nil()), |tail, x| tr(t(tag(1)), x, tail))
}
fn rec(tag_: u8, xs: Vec<E>) -> E {
    tr(t(tag(tag_)), ls(xs), t(nil()))
}
fn lower(e: &E, env: &[&str]) -> KExpr {
    match e {
        E::B(x) => KExpr::BytesLiteral(x.clone()),
        E::T(x) => KExpr::TermLiteral(clone_term(x)),
        E::V(n) => KExpr::Var(
            env.iter()
                .position(|x| x == n)
                .unwrap_or_else(|| panic!("unbound {n} in {env:?}")) as u32,
        ),
        E::Atom(a, b, c) => KExpr::MakeAtom {
            kind: bx(lower(a, env)),
            payload: bx(lower(b, env)),
            equality: bx(lower(c, env)),
        },
        E::Triple(a, b, c) => KExpr::MakeTriple {
            first: bx(lower(a, env)),
            second: bx(lower(b, env)),
            third: bx(lower(c, env)),
        },
        E::Let(n, a, c) => {
            let mut ns = vec![*n];
            ns.extend_from_slice(env);
            KExpr::Let {
                value: bx(lower(a, env)),
                body: bx(lower(c, &ns)),
            }
        }
        E::Case(a, ns, b, c) => {
            let mut vs = ns.to_vec();
            vs.extend_from_slice(env);
            KExpr::CaseTerm {
                scrutinee: bx(lower(a, env)),
                atom_body: bx(lower(b, &vs)),
                triple_body: bx(lower(c, &vs)),
            }
        }
        E::Byte(a, ns, b, c) => {
            let mut vs = ns.to_vec();
            vs.extend_from_slice(env);
            KExpr::CaseBytes {
                scrutinee: bx(lower(a, env)),
                empty_body: bx(lower(b, env)),
                cons_body: bx(lower(c, &vs)),
            }
        }
        E::Cat(xs) => KExpr::ConcatBytes(xs.iter().map(|x| lower(x, env)).collect()),
        E::Eq(a, b, c, d) => KExpr::CaseBytesEqual {
            left: bx(lower(a, env)),
            right: bx(lower(b, env)),
            equal_body: bx(lower(c, env)),
            unequal_body: bx(lower(d, env)),
        },
        E::Call(n, xs) => KExpr::Call {
            definition_id: ident(*n),
            arguments: xs.iter().map(|x| lower(x, env)).collect(),
        },
        E::Sha(x) => KExpr::Request {
            physical_operation_id: sha256_operation_id(),
            arguments: vec![lower(x, env)],
        },
    }
}
const COMPILE: u32 = 1;
const ADMIT: u32 = 2;
const PAYLOAD: u32 = 3;
const FIRST: u32 = 4;
const SECOND: u32 = 5;
const THIRD: u32 = 6;
const INC: u32 = 8;
const LENGTH: u32 = 9;
const LENLOOP: u32 = 10;
const HEXBYTE: u32 = 11;
const LOOKUP: u32 = 12;
const UNHEX: u32 = 13;
const BLOB: u32 = 14;
const ENCTERM: u32 = 15;
const READBLOB: u32 = 16;
const READTERM: u32 = 17;
const ENCEXPR: u32 = 18;
const ENCEXPRS: u32 = 19;
const LISTLEN: u32 = 20;
const LISTLENLOOP: u32 = 21;
const ENCDEF: u32 = 22;
const ENCDEFS: u32 = 23;
const ENCDECL: u32 = 24;
const ENCDECLS: u32 = 25;
const ANALYZE: u32 = 26;
const CONSTITUTE: u32 = 34;
fn payload(x: E) -> E {
    call(PAYLOAD, vec![x])
}
fn first(x: E) -> E {
    call(FIRST, vec![x])
}
fn second(x: E) -> E {
    call(SECOND, vec![x])
}
fn third(x: E) -> E {
    call(THIRD, vec![x])
}
fn field(x: E, i: usize) -> E {
    let mut fields = second(x);
    for _ in 0..i {
        fields = third(fields)
    }
    second(fields)
}
fn definition(id: u32, args: &[(&'static str, KSort)], result: KSort, body: E) -> Definition {
    Definition {
        id: ident(id),
        arguments: args.iter().map(|a| a.1).collect(),
        result,
        body: lower(&body, &args.iter().map(|a| a.0).collect::<Vec<_>>()),
    }
}
fn def(id: u32, args: &[(&'static str, KSort)], result: KSort, body: E, p: &mut Vec<Definition>) {
    p.push(definition(id, args, result, body))
}
fn tree(rows: &[(Vec<u8>, Term)]) -> Term {
    if rows.is_empty() {
        nil()
    } else {
        let mid = rows.len() / 2;
        triple(
            triple(bytes(&rows[mid].0), clone_term(&rows[mid].1), nil()),
            tree(&rows[..mid]),
            tree(&rows[mid + 1..]),
        )
    }
}
fn core() -> Vec<Definition> {
    use KSort::{Bytes as B, Term as T};
    let mut p = vec![];
    def(
        PAYLOAD,
        &[("x", T)],
        B,
        ct(v("x"), ["k", "p", "e"], v("p"), b([])),
        &mut p,
    );
    for (id, n) in [(FIRST, "a"), (SECOND, "b"), (THIRD, "c")] {
        def(
            id,
            &[("x", T)],
            T,
            ct(v("x"), ["a", "b", "c"], t(nil()), v(n)),
            &mut p,
        )
    }
    def(
        LOOKUP,
        &[("key", B), ("tree", T)],
        T,
        ct(
            v("tree"),
            ["node", "left", "right"],
            t(nil()),
            eq(
                payload(first(v("node"))),
                v("key"),
                second(v("node")),
                let_(
                    "hit",
                    call(LOOKUP, vec![v("key"), v("left")]),
                    eq(
                        payload(first(v("hit"))),
                        b([0]),
                        call(LOOKUP, vec![v("key"), v("right")]),
                        v("hit"),
                    ),
                ),
            ),
        ),
        &mut p,
    );
    // Lookup results are wrapped so an octet zero is distinct from absence.
    let high = (0u8..16)
        .map(|h| {
            let low = (0u8..16)
                .map(|l| {
                    (
                        vec![b"0123456789abcdef"[l as usize]],
                        triple(bytes(b"hit"), bytes(&[h * 16 + l]), nil()),
                    )
                })
                .collect::<Vec<_>>();
            (
                vec![b"0123456789abcdef"[h as usize]],
                triple(bytes(b"hit"), tree(&low), nil()),
            )
        })
        .collect::<Vec<_>>();
    def(
        HEXBYTE,
        &[("h", B), ("l", B)],
        B,
        payload(second(call(
            LOOKUP,
            vec![v("l"), second(call(LOOKUP, vec![v("h"), t(tree(&high))]))],
        ))),
        &mut p,
    );
    let mut increment = b([]);
    for i in (0usize..15).rev() {
        increment = eq(
            v("h"),
            b([b"0123456789abcdef"[i]]),
            cat(vec![b([b"0123456789abcdef"[i + 1]]), v("tail")]),
            increment,
        )
    }
    def(
        INC,
        &[("digits", B)],
        B,
        cb(
            v("digits"),
            ["h", "tail"],
            b([]),
            eq(
                v("h"),
                b(b"f"),
                cat(vec![b(b"0"), call(INC, vec![v("tail")])]),
                increment,
            ),
        ),
        &mut p,
    );
    def(
        UNHEX,
        &[("digits", B)],
        B,
        cb(
            v("digits"),
            ["low", "rest"],
            b([]),
            cb(
                v("rest"),
                ["high", "tail"],
                b([]),
                cat(vec![
                    call(UNHEX, vec![v("tail")]),
                    call(HEXBYTE, vec![v("high"), v("low")]),
                ]),
            ),
        ),
        &mut p,
    );
    def(
        LENLOOP,
        &[("input", B), ("count", B)],
        B,
        cb(
            v("input"),
            ["head", "tail"],
            call(UNHEX, vec![v("count")]),
            call(LENLOOP, vec![v("tail"), call(INC, vec![v("count")])]),
        ),
        &mut p,
    );
    def(
        LENGTH,
        &[("input", B)],
        B,
        call(LENLOOP, vec![v("input"), b(b"00000000")]),
        &mut p,
    );
    def(
        LISTLENLOOP,
        &[("list", T), ("count", B)],
        B,
        ct(
            v("list"),
            ["mark", "head", "tail"],
            call(UNHEX, vec![v("count")]),
            call(LISTLENLOOP, vec![v("tail"), call(INC, vec![v("count")])]),
        ),
        &mut p,
    );
    def(
        LISTLEN,
        &[("list", T)],
        B,
        call(LISTLENLOOP, vec![v("list"), b(b"00000000")]),
        &mut p,
    );
    def(
        BLOB,
        &[("input", B)],
        B,
        cat(vec![call(LENGTH, vec![v("input")]), v("input")]),
        &mut p,
    );
    def(
        ENCTERM,
        &[("x", T)],
        B,
        ct(
            v("x"),
            ["a", "b", "c"],
            cat(vec![
                b([0]),
                call(BLOB, vec![v("a")]),
                call(BLOB, vec![v("b")]),
                call(BLOB, vec![v("c")]),
            ]),
            cat(vec![
                b([1]),
                call(ENCTERM, vec![v("a")]),
                call(ENCTERM, vec![v("b")]),
                call(ENCTERM, vec![v("c")]),
            ]),
        ),
        &mut p,
    );
    let fail = tr(t(nil()), tb(b([])), tb(b(b"error")));
    let prefix_blob = tr(
        tb(cat(vec![v("octet"), payload(first(v("rest")))])),
        second(v("rest")),
        third(v("rest")),
    );
    let recurse_blob = let_("rest", call(READBLOB, vec![v("tail2")]), prefix_blob);
    let pair_blob = let_(
        "octet",
        call(HEXBYTE, vec![v("h"), v("l")]),
        eq(v("octet"), b([]), fail.clone(), recurse_blob),
    );
    let tail_blob = cb(v("tail"), ["l", "tail2"], fail.clone(), pair_blob);
    let finish_blob = tr(tb(b([])), tb(v("tail")), tb(b(b"ok")));
    let read_blob = cb(
        v("input"),
        ["h", "tail"],
        fail.clone(),
        eq(v("h"), b(b"."), finish_blob, tail_blob),
    );
    def(READBLOB, &[("input", B)], T, read_blob, &mut p);
    let atom_body = let_(
        "kind",
        call(READBLOB, vec![v("tail")]),
        let_(
            "value",
            call(READBLOB, vec![payload(second(v("kind")))]),
            let_(
                "equality",
                call(READBLOB, vec![payload(second(v("value")))]),
                tr(
                    at(
                        payload(first(v("kind"))),
                        payload(first(v("value"))),
                        payload(first(v("equality"))),
                    ),
                    second(v("equality")),
                    third(v("equality")),
                ),
            ),
        ),
    );
    let triple_body = let_(
        "a",
        call(READTERM, vec![v("tail")]),
        let_(
            "b",
            call(READTERM, vec![payload(second(v("a")))]),
            let_(
                "c",
                call(READTERM, vec![payload(second(v("b")))]),
                tr(
                    tr(first(v("a")), first(v("b")), first(v("c"))),
                    second(v("c")),
                    third(v("c")),
                ),
            ),
        ),
    );
    let short_atom = |kind: &[u8]| {
        let_(
            "blob",
            call(READBLOB, vec![v("tail")]),
            tr(
                at(b(kind), payload(first(v("blob"))), b(EQ)),
                second(v("blob")),
                third(v("blob")),
            ),
        )
    };
    let short_list = let_(
        "a",
        call(READTERM, vec![v("tail")]),
        let_(
            "b",
            call(READTERM, vec![payload(second(v("a")))]),
            tr(
                tr(t(tag(1)), first(v("a")), first(v("b"))),
                second(v("b")),
                third(v("b")),
            ),
        ),
    );
    let short_record = let_(
        "tag",
        call(READBLOB, vec![v("tail")]),
        let_(
            "fields",
            call(READTERM, vec![payload(second(v("tag")))]),
            tr(
                tr(
                    at(b(TAG), payload(first(v("tag"))), b(EQ)),
                    first(v("fields")),
                    t(nil()),
                ),
                second(v("fields")),
                third(v("fields")),
            ),
        ),
    );
    let mut reading = fail.clone();
    for (token, body) in [
        (b'a', atom_body),
        (b't', triple_body),
        (b'b', short_atom(BYTES)),
        (b'g', short_atom(TAG)),
        (b'n', tr(t(nil()), tb(v("tail")), tb(b(b"ok")))),
        (b'l', short_list),
        (b'r', short_record),
    ] {
        reading = eq(v("h"), b([token]), body, reading);
    }
    def(
        READTERM,
        &[("input", B)],
        T,
        cb(v("input"), ["h", "tail"], fail.clone(), reading),
        &mut p,
    );
    p
}
fn compiler(p: &mut Vec<Definition>) {
    use KSort::{Bytes as B, Term as T};
    def(
        ENCEXPRS,
        &[("xs", T)],
        B,
        ct(
            v("xs"),
            ["mark", "head", "tail"],
            b([]),
            cat(vec![
                call(ENCEXPR, vec![v("head")]),
                call(ENCEXPRS, vec![v("tail")]),
            ]),
        ),
        p,
    );
    let mut expr = b([0xff]);
    for code in (0u8..12).rev() {
        let fields = second(v("expr"));
        let children = |count: usize| {
            (0..count)
                .map(|n| call(ENCEXPR, vec![field(v("expr"), n)]))
                .collect::<Vec<_>>()
        };
        let mut parts = vec![b([code])];
        match code {
            0 => parts.push(call(BLOB, vec![payload(field(v("expr"), 0))])),
            1 => parts.push(call(ENCTERM, vec![field(v("expr"), 0)])),
            2 => parts.push(payload(field(v("expr"), 0))),
            3 | 4 | 6 | 7 => parts.extend(children(3)),
            5 => parts.extend(children(2)),
            8 => {
                parts.push(call(LISTLEN, vec![fields.clone()]));
                parts.push(call(ENCEXPRS, vec![fields]));
            }
            9 => parts.extend(children(4)),
            10 | 11 => {
                parts.push(payload(field(v("expr"), 0)));
                parts.push(call(LISTLEN, vec![field(v("expr"), 1)]));
                parts.push(call(ENCEXPRS, vec![field(v("expr"), 1)]));
            }
            _ => unreachable!(),
        };
        expr = eq(payload(first(v("expr"))), b([code]), cat(parts), expr)
    }
    def(ENCEXPR, &[("expr", T)], B, expr, p);
    def(
        ENCDEF,
        &[("definition", T)],
        B,
        cat(vec![
            payload(field(v("definition"), 0)),
            payload(field(v("definition"), 1)),
            payload(field(v("definition"), 2)),
            call(ENCEXPR, vec![field(v("definition"), 3)]),
        ]),
        p,
    );
    def(
        ENCDEFS,
        &[("definitions", T)],
        B,
        ct(
            v("definitions"),
            ["m", "head", "tail"],
            b([]),
            cat(vec![
                call(ENCDEF, vec![v("head")]),
                call(ENCDEFS, vec![v("tail")]),
            ]),
        ),
        p,
    );
    def(
        ENCDECL,
        &[("declaration", T), ("revision", B)],
        B,
        cat(vec![
            payload(field(v("declaration"), 0)),
            payload(field(v("declaration"), 1)),
            payload(field(v("declaration"), 2)),
            eq(
                payload(field(v("declaration"), 0)),
                b([1]),
                v("revision"),
                b([]),
            ),
        ]),
        p,
    );
    def(
        ENCDECLS,
        &[("declarations", T), ("revision", B)],
        B,
        ct(
            v("declarations"),
            ["m", "head", "tail"],
            b([]),
            cat(vec![
                call(ENCDECL, vec![v("head"), v("revision")]),
                call(ENCDECLS, vec![v("tail"), v("revision")]),
            ]),
        ),
        p,
    );
    let build = let_(
        "source",
        field(second(field(v("request"), 4)), 2),
        let_(
            "parsed",
            call(READTERM, vec![payload(v("source"))]),
            eq(
                payload(third(v("parsed"))),
                b(b"ok"),
                eq(
                    payload(second(v("parsed"))),
                    b([]),
                    let_(
                        "subject",
                        first(v("parsed")),
                        let_(
                            "base",
                            field(v("request"), 0),
                            rec(
                                0x14,
                                vec![tb(cat(vec![
                                    eq(
                                        payload(first(v("base"))),
                                        b([0x10]),
                                        b([0]),
                                        cat(vec![
                                            b([1]),
                                            payload(field(v("base"), 0)),
                                            payload(field(v("request"), 7)),
                                        ]),
                                    ),
                                    call(LISTLEN, vec![field(v("subject"), 0)]),
                                    call(
                                        ENCDECLS,
                                        vec![field(v("subject"), 0), payload(field(v("base"), 1))],
                                    ),
                                    payload(field(v("subject"), 1)),
                                    call(LISTLEN, vec![field(v("subject"), 2)]),
                                    call(ENCDEFS, vec![field(v("subject"), 2)]),
                                    call(ENCTERM, vec![v("request")]),
                                ]))],
                            ),
                        ),
                    ),
                    rec(0x15, vec![tb(b(b"malformed source"))]),
                ),
                rec(0x15, vec![tb(b(b"malformed source"))]),
            ),
        ),
    );
    def(COMPILE, &[("request", T)], T, build, p);
    def(
        ADMIT,
        &[("proposal", T)],
        T,
        let_(
            "compiled",
            call(COMPILE, vec![field(v("proposal"), 0)]),
            eq(
                payload(first(v("compiled"))),
                b([0x14]),
                eq(
                    payload(field(v("compiled"), 0)),
                    payload(field(v("proposal"), 1)),
                    rec(0x17, vec![field(v("proposal"), 1)]),
                    rec(0x18, vec![tb(b(b"subject differs"))]),
                ),
                rec(0x18, vec![tb(b(b"source rejected"))]),
            ),
        ),
        p,
    );
    def(
        CONSTITUTE,
        &[("request", T)],
        B,
        compiler_process_expression(),
        p,
    );
}
fn reify(e: &KExpr) -> Term {
    match e {
        KExpr::BytesLiteral(x) => record(0, vec![bytes(x)]),
        KExpr::TermLiteral(x) => record(1, vec![clone_term(x)]),
        KExpr::Var(x) => record(2, vec![bytes(&x.to_be_bytes())]),
        KExpr::MakeAtom {
            kind,
            payload,
            equality,
        } => record(3, vec![reify(kind), reify(payload), reify(equality)]),
        KExpr::MakeTriple {
            first,
            second,
            third,
        } => record(4, vec![reify(first), reify(second), reify(third)]),
        KExpr::Let { value, body } => record(5, vec![reify(value), reify(body)]),
        KExpr::CaseTerm {
            scrutinee,
            atom_body,
            triple_body,
        } => record(
            6,
            vec![reify(scrutinee), reify(atom_body), reify(triple_body)],
        ),
        KExpr::CaseBytes {
            scrutinee,
            empty_body,
            cons_body,
        } => record(
            7,
            vec![reify(scrutinee), reify(empty_body), reify(cons_body)],
        ),
        KExpr::ConcatBytes(xs) => record(8, xs.iter().map(reify).collect()),
        KExpr::CaseBytesEqual {
            left,
            right,
            equal_body,
            unequal_body,
        } => record(
            9,
            vec![
                reify(left),
                reify(right),
                reify(equal_body),
                reify(unequal_body),
            ],
        ),
        KExpr::Call {
            definition_id,
            arguments,
        } => record(
            10,
            vec![
                bytes(definition_id.as_bytes()),
                list(arguments.iter().map(reify).collect()),
            ],
        ),
        KExpr::Request {
            physical_operation_id,
            arguments,
        } => record(
            11,
            vec![
                bytes(physical_operation_id.as_bytes()),
                list(arguments.iter().map(reify).collect()),
            ],
        ),
    }
}
fn source_term(p: &[Definition], declarations: &[NominalDeclaration]) -> Term {
    let declarations = declarations
        .iter()
        .map(|d| match d {
            NominalDeclaration::Seed { domain, id } => record(
                0x41,
                vec![bytes(&[0]), bytes(domain.as_bytes()), bytes(id.as_bytes())],
            ),
            NominalDeclaration::RetainedSeed { domain, id, .. } => record(
                0x41,
                vec![bytes(&[1]), bytes(domain.as_bytes()), bytes(id.as_bytes())],
            ),
            _ => unreachable!(),
        })
        .collect();
    let definitions = p
        .iter()
        .map(|d| {
            let mut args = (d.arguments.len() as u32).to_be_bytes().to_vec();
            args.extend(
                d.arguments
                    .iter()
                    .map(|s| if *s == KSort::Bytes { 0 } else { 1 }),
            );
            record(
                0x42,
                vec![
                    bytes(d.id.as_bytes()),
                    bytes(&args),
                    bytes(&[if d.result == KSort::Bytes { 0 } else { 1 }]),
                    reify(&d.body),
                ],
            )
        })
        .collect();
    let mut interface = ident(COMPILE).0.to_vec();
    interface.extend(ident(ADMIT).0);
    record(
        0x40,
        vec![list(declarations), bytes(&interface), list(definitions)],
    )
}
fn term_depth(term: &Term) -> usize {
    match term {
        Term::Atom { .. } => 1,
        Term::Triple(a, b, c) => 1 + term_depth(a).max(term_depth(b)).max(term_depth(c)),
    }
}
fn textual(term: &Term, out: &mut Vec<u8>) {
    fn hex(x: &[u8], out: &mut Vec<u8>) {
        for byte in x {
            out.push(b"0123456789abcdef"[(byte >> 4) as usize]);
            out.push(b"0123456789abcdef"[(byte & 15) as usize]);
        }
        out.push(b'.');
    }
    match term {
        Term::Atom {
            kind,
            canonical_payload,
            equality_contract,
        } if equality_contract == EQ && (kind == BYTES || kind == TAG) => {
            if kind == TAG && canonical_payload == &[0] {
                out.push(b'n');
                return;
            }
            out.push(if kind == BYTES { b'b' } else { b'g' });
            hex(canonical_payload, out);
            return;
        }
        Term::Triple(a, b, c) => {
            if let Term::Atom {
                kind,
                canonical_payload,
                equality_contract,
            } = &**a
            {
                if kind == TAG && equality_contract == EQ {
                    if canonical_payload == &[1] {
                        out.push(b'l');
                        textual(b, out);
                        textual(c, out);
                        return;
                    }
                    if **c == nil() {
                        out.push(b'r');
                        hex(canonical_payload, out);
                        textual(b, out);
                        return;
                    }
                }
            }
        }
        _ => {}
    }
    match term {
        Term::Atom {
            kind,
            canonical_payload,
            equality_contract,
        } => {
            out.push(b'a');
            for blob in [kind, canonical_payload, equality_contract] {
                for byte in blob {
                    out.push(b"0123456789abcdef"[(byte >> 4) as usize]);
                    out.push(b"0123456789abcdef"[(byte & 15) as usize]);
                }
                out.push(b'.')
            }
        }
        Term::Triple(a, b, c) => {
            out.push(b't');
            textual(a, out);
            textual(b, out);
            textual(c, out)
        }
    }
}
fn domain(component: &[u8]) -> Id32 {
    Id32(domain_hash("clause/nominal-domain/v1", &[component]).0)
}
fn declarations(p: &[Definition]) -> Vec<NominalDeclaration> {
    let mut rows = p
        .iter()
        .map(|d| NominalDeclaration::Seed {
            domain: domain(b"definition"),
            id: d.id,
        })
        .collect::<Vec<_>>();
    rows.push(NominalDeclaration::Seed {
        domain: domain(b"source-unit"),
        id: ident(200),
    });
    rows.push(NominalDeclaration::Seed {
        domain: domain(b"change-occurrence"),
        id: ident(201),
    });
    for index in 300..305 {
        rows.push(NominalDeclaration::Seed {
            domain: domain(b"referent"),
            id: ident(index),
        });
    }
    rows.sort_by_key(|d| match d {
        NominalDeclaration::Seed { domain, id } => (*domain, *id),
        _ => unreachable!(),
    });
    rows
}
fn request(
    declarations: &[NominalDeclaration],
    source: &[u8],
    predecessor: Option<(Hash32, Id32)>,
    change: Id32,
) -> Term {
    let mut retained = vec![];
    let mut seeds = vec![];
    for d in declarations {
        let (domain, id, retain) = match d {
            NominalDeclaration::Seed { domain, id } => (*domain, *id, false),
            NominalDeclaration::RetainedSeed { domain, id, .. } => (*domain, *id, true),
            _ => unreachable!(),
        };
        let reference = record(0x04, vec![idterm(domain), idterm(id)]);
        if retain {
            retained.push(record(0x09, vec![reference]))
        } else {
            seeds.push(record(0x0a, vec![reference]))
        }
    }
    record(
        0x13,
        vec![
            match predecessor {
                None => record(0x10, vec![]),
                Some((hash, rev)) => record(0x11, vec![idterm(Id32(hash.0)), idterm(rev)]),
            },
            idterm(Id32(core_contract_id().unwrap().0)),
            idterm(Id32(physical_profile_id().unwrap().0)),
            bytes(b"compiler"),
            list(vec![record(
                0x12,
                vec![
                    idterm(ident(200)),
                    idterm(Id32(source_artifact_id(source).0)),
                    bytes(source),
                ],
            )]),
            nil(),
            record(0x08, vec![list(retained), list(seeds)]),
            idterm(change),
            nil(),
            number(1_000_000_000),
            number(1_000_000_000),
            list(vec![]),
        ],
    )
}
fn main() {
    let mut program = core();
    compiler(&mut program);
    language(&mut program, false);
    program.sort_by_key(|d| d.id);
    let declarations = declarations(&program);
    let mut source = vec![];
    textual(&source_term(&program, &declarations), &mut source);
    let build_request = request(&declarations, &source, None, ident(201));
    let package = CompilerPackage {
        core_manifest: CoreManifest::canonical_v1(),
        subject: CompilerSubject {
            lineage: CompilerLineage::Genesis,
            nominal_declarations: declarations,
            interface: CompilerInterface {
                compile: ident(COMPILE),
                admit_propose: ident(ADMIT),
            },
            program,
            build_request,
        },
        evidence: CompilerEvidence::Genesis,
    };
    let wire = encode(&package).unwrap();
    let output =
        std::env::var("FREEZE_OUTPUT_ROOT").unwrap_or_else(|_| "target/host-freeze".into());
    let root = Path::new(&output);
    fs::create_dir_all(root).unwrap();
    fs::write(root.join("compiler0.clcp"), &wire).unwrap();
    fs::write(root.join("compiler0.source"), &source).unwrap();
    fs::write(
        root.join("build-request.term"),
        encode_canonical_term(&package.subject.build_request).unwrap(),
    )
    .unwrap();
    println!("source={} package={}", source.len(), wire.len());
    let e = Evaluator::new(&package.subject.program).unwrap();
    for sample in [
        bytes(b"hello"),
        record(0x55, vec![bytes(b"name"), record(0x20, vec![])]),
        tup_sample(),
    ] {
        let mut text = vec![];
        textual(&sample, &mut text);
        let parsed = e
            .invoke_entrypoint(ident(READTERM), &[KValue::Bytes(text)], 1_000_000)
            .unwrap();
        let KValue::Term(Term::Triple(actual, remainder, status)) = parsed.value else {
            panic!("reader shape")
        };
        assert_eq!(*actual, sample, "source reader preserves the complete Term");
        assert_eq!(*remainder, bytes(b""));
        assert_eq!(*status, bytes(b"ok"));
        let encoded = e
            .invoke_entrypoint(ident(ENCTERM), &[KValue::Term(sample)], 1_000_000)
            .unwrap();
        let KValue::Bytes(encoded) = encoded.value else {
            panic!("encoder sort")
        };
        assert_eq!(encoded, encode_canonical_term(&actual).unwrap());
    }
    println!("focused neutral source reader and canonical Term encoder passed");
    fs::write(root.join("constitute.id"), ident(CONSTITUTE).0).unwrap();
    let outer = e
        .invoke_entrypoint(
            ident(CONSTITUTE),
            &[KValue::Term(record(
                0x99,
                vec![bytes(&wire), bytes(b"build request"), bytes(&[0; 4])],
            ))],
            1_000_000_000,
        )
        .unwrap();
    let KValue::Bytes(ref outer_bytes) = outer.value else {
        panic!("constitution result sort")
    };
    let outer = clause_package::check_process_package(
        clause_package::decode_process_package(outer_bytes).unwrap(),
    )
    .unwrap();
    assert_eq!(
        outer
            .constitution()
            .formation(clause_package::FormationLocalId::new(1))
            .unwrap()
            .term
            .as_atom()
            .unwrap()
            .canonical_payload(),
        wire
    );
    println!(
        "Compiler0 constructs a checked compiler Application constitution from exact supplied compiler and request bytes"
    );
    let mut program1 = core();
    compiler(&mut program1);
    language(&mut program1, true);
    program1.sort_by_key(|d| d.id);
    language_specs(root, &package.subject.program, &program1);
    if std::env::var_os("MATERIALIZE_ONLY").is_some() {
        return;
    }
    let predecessor = decode(&wire).unwrap();
    let predecessor_hash = compiler_package_hash(&wire);
    let predecessor_revision = Id32(
        domain_hash(
            "clause/compiler-revision/v1",
            &[predecessor.exact_subject()],
        )
        .0,
    );
    let changed = package
        .subject
        .program
        .iter()
        .zip(&program1)
        .filter_map(|(zero, one)| {
            assert_eq!(zero.id, one.id);
            assert_eq!(zero.arguments, one.arguments);
            assert_eq!(zero.result, one.result);
            (zero.body != one.body).then_some(zero.id)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        changed,
        vec![ident(DIAG), ident(BIND), ident(EFFECT), ident(MACRO)]
    );
    let mut declarations1 = package
        .subject
        .nominal_declarations
        .iter()
        .map(|d| {
            let NominalDeclaration::Seed { domain, id } = d else {
                unreachable!()
            };
            NominalDeclaration::RetainedSeed {
                domain: *domain,
                id: *id,
                predecessor_revision_id: predecessor_revision,
            }
        })
        .collect::<Vec<_>>();
    declarations1.push(NominalDeclaration::Seed {
        domain: domain(b"change-occurrence"),
        id: ident(202),
    });
    declarations1.sort_by_key(|d| match d {
        NominalDeclaration::Seed { domain, id }
        | NominalDeclaration::RetainedSeed { domain, id, .. } => (*domain, *id),
        _ => unreachable!(),
    });
    let mut source1 = vec![];
    let syntax1 = source_term(&program1, &declarations1);
    println!(
        "Compiler1 source Term depth {}; canonical encoding {:?}",
        term_depth(&syntax1),
        encode_canonical_term(&syntax1).map(|b| b.len())
    );
    textual(&syntax1, &mut source1);
    let build1 = request(
        &declarations1,
        &source1,
        Some((predecessor_hash, predecessor_revision)),
        ident(202),
    );
    let empty_receipt = EvalReceipt {
        format_version: 0,
        expected_value_hash: Hash32([0; 32]),
        expected_remaining_fuel: 0,
        expected_observations_hash: Hash32([0; 32]),
    };
    let mut candidate = CompilerPackage {
        core_manifest: CoreManifest::canonical_v1(),
        subject: CompilerSubject {
            lineage: CompilerLineage::Successor {
                predecessor_locator: predecessor_hash,
                change_occurrence_id: ident(202),
            },
            nominal_declarations: declarations1,
            interface: CompilerInterface {
                compile: ident(COMPILE),
                admit_propose: ident(ADMIT),
            },
            program: program1,
            build_request: build1,
        },
        evidence: CompilerEvidence::Successor {
            compile_receipt: empty_receipt,
            admission_receipt: empty_receipt,
        },
    };
    let subject = decode(&encode(&candidate).unwrap())
        .unwrap()
        .exact_subject()
        .to_vec();
    fs::write(root.join("compiler1.source"), &source1).unwrap();
    fs::write(
        root.join("compiler1.request.term"),
        encode_canonical_term(&candidate.subject.build_request).unwrap(),
    )
    .unwrap();
    fs::write(root.join("compile.id"), ident(COMPILE).0).unwrap();
    fs::write(root.join("admit.id"), ident(ADMIT).0).unwrap();
    fs::write(root.join("analyze.id"), ident(ANALYZE).0).unwrap();
    fs::write(
        root.join("compiler1.candidate.clcp"),
        encode(&candidate).unwrap(),
    )
    .unwrap();
    fs::write(root.join("compiler1.subject"), &subject).unwrap();
    if std::env::var_os("MATERIALIZE_CANDIDATE_ONLY").is_some() {
        return;
    }
    let compiled = e
        .invoke_entrypoint(
            ident(COMPILE),
            &[KValue::Term(clone_term(&candidate.subject.build_request))],
            1_000_000_000,
        )
        .unwrap();
    let KValue::Term(ref result) = compiled.value else {
        panic!("compile result sort")
    };
    assert_eq!(data(parts(result).0), [0x14]);
    assert_eq!(data(at_field(result, 0)), subject);
    write_eval(root, "compiler1.compile", &compiled);
    println!(
        "Compiler0 compiled full Compiler1 subject; fuel {}",
        compiled.remaining_fuel
    );
    let observations = compiled.observations.try_to_term().unwrap();
    let proposal = record(
        0x16,
        vec![
            clone_term(&candidate.subject.build_request),
            bytes(&subject),
            observations,
        ],
    );
    fs::write(
        root.join("compiler1.proposal.term"),
        encode_canonical_term(&proposal).unwrap(),
    )
    .unwrap();
    let proposed = e
        .invoke_entrypoint(ident(ADMIT), &[KValue::Term(proposal)], 1_000_000_000)
        .unwrap();
    let KValue::Term(ref result) = proposed.value else {
        panic!("proposal result sort")
    };
    assert_eq!(data(parts(result).0), [0x17]);
    assert_eq!(data(at_field(result, 0)), subject);
    write_eval(root, "compiler1.propose", &proposed);
    println!(
        "Compiler0 proposed exact Compiler1 subject; fuel {}",
        proposed.remaining_fuel
    );
    let receipt = |evaluation: &clause_substrate::evaluator::Evaluation| EvalReceipt {
        format_version: 0,
        expected_value_hash: eval_receipt_value_hash(&evaluation.value).unwrap(),
        expected_remaining_fuel: evaluation.remaining_fuel,
        expected_observations_hash: eval_receipt_observations_hash(
            &evaluation.observations.try_to_term().unwrap(),
        )
        .unwrap(),
    };
    candidate.evidence = CompilerEvidence::Successor {
        compile_receipt: receipt(&compiled),
        admission_receipt: receipt(&proposed),
    };
    let candidate_wire = encode(&candidate).unwrap();
    assert_eq!(decode(&candidate_wire).unwrap().exact_subject(), subject);
    fs::write(root.join("compiler1.clcp"), candidate_wire).unwrap();
    println!("Compiler1 receipts attached without subject mutation");
}
fn tup_sample() -> Term {
    triple(bytes(b"left"), bytes(b"middle"), bytes(b"right"))
}

const CHECK: u32 = 27;
const FIND: u32 = 28;
const NEWID: u32 = 29;
const DIAG: u32 = 30;
const BIND: u32 = 31;
const EFFECT: u32 = 32;
const MACRO: u32 = 33;
fn effect_process_package(message: &[u8]) -> clause_package::CheckedProcessPackage {
    use clause_package as q;
    let scope = q::TermScope {
        universe: q::UniverseId::from_bytes(ident(700).0),
        semantics: q::ClauseSemanticsId::from_bytes(ident(701).0),
    };
    let atom = |payload: &[u8]| {
        q::Term::atom(
            scope,
            b"compiler-language/bytes".to_vec(),
            payload.to_vec(),
            q::EqualityContract::ExactOctetsV1,
        )
        .unwrap()
    };
    let target = q::FormationTargetV2 {
        type_term: atom(b"Text"),
        interpretation: atom(b"exact octets"),
    };
    let one = q::CardinalityV2 {
        minimum: 1,
        maximum: Some(1),
    };
    let sid = q::RelationSchemaLocalId::new(1);
    let op = q::OperatorLocalId::new(1);
    let mode = q::ModeLocalId::new(1);
    let checker = q::ModeLocalId::new(2);
    let capability = q::CapabilityLocalId::new(1);
    let mut dependencies = (1..=5)
        .map(|i| q::LocalSemanticDependencyV2::Formation(q::FormationLocalId::new(i)))
        .collect::<Vec<_>>();
    dependencies.push(q::LocalSemanticDependencyV2::RelationSchema(sid));
    dependencies.extend((1..=3).map(|i| {
        q::LocalSemanticDependencyV2::Role(q::LocalRoleRefV2 {
            schema: sid,
            role: q::RoleLocalId::new(i),
        })
    }));
    dependencies.push(q::LocalSemanticDependencyV2::Operator(op));
    dependencies.extend(
        [mode, checker].map(|mode| {
            q::LocalSemanticDependencyV2::Mode(q::LocalModeRefV2 { operator: op, mode })
        }),
    );
    dependencies.push(q::LocalSemanticDependencyV2::Capability(capability));
    dependencies.sort();
    let mut formations = [b"notify".as_slice(), b"outbox", message, b"mailbox"]
        .into_iter()
        .enumerate()
        .map(|(i, value)| q::FormationJudgmentPreimageV2 {
            id: q::FormationLocalId::new((i + 1) as u32),
            context: vec![],
            term: atom(value),
            target: target.clone(),
            direct_dependencies: vec![],
        })
        .collect::<Vec<_>>();
    formations.push(q::FormationJudgmentPreimageV2 {
        id: q::FormationLocalId::new(5),
        context: vec![],
        term: atom(b"dispatch"),
        target: target.clone(),
        direct_dependencies: dependencies
            .iter()
            .filter(|d| **d != q::LocalSemanticDependencyV2::Formation(q::FormationLocalId::new(5)))
            .cloned()
            .collect(),
    });
    let common_contract = q::ModeContractV2 {
        determinism: q::DeterminismContractV2::Deterministic,
        result_cardinality: one,
        result_order: q::ResultOrderContractV2::UnorderedFiniteSet,
        failure_domain: None,
        state_delta_domain: Some(target.clone()),
        budget_exhaustion_domain: None,
        effect_intents: vec![q::EffectIntentContractPreimageV2 {
            intent_domain: target.clone(),
            action_role: q::RoleLocalId::new(1),
            resource_role: q::RoleLocalId::new(2),
            payload_role: q::RoleLocalId::new(3),
            required_capability: capability,
        }],
        formation_checks: vec![target.clone()],
        productivity: q::ProductivityContractV2 {
            kind: q::ProductivityKindV2::Partial,
            obligations: vec![],
        },
        scheduling_requirements: vec![],
        resource_requirements: vec![],
        capability_requirements: vec![capability],
        continuation: q::ContinuationContractV2::TerminalOnly { may_cancel: false },
    };
    let mut checker_contract = common_contract.clone();
    checker_contract.state_delta_domain = None;
    checker_contract.effect_intents.clear();
    checker_contract.formation_checks = vec![target.clone()];
    let make_mode = |id, contract| q::ModePreimageV2 {
        id,
        schema: sid,
        known_roles: (1..=3).map(q::RoleLocalId::new).collect(),
        produced_roles: vec![],
        static_basis: q::StaticActivationBasisPreimageV2 {
            context_requirements: vec![],
            constitutive_dependencies: vec![],
        },
        authorization_requirements: vec![],
        dynamic_prerequisites: vec![],
        contract,
        direct_dependencies: vec![],
    };
    let snapshot = q::ProgramSnapshotPreimageV2 {
        constitution: q::ProgramConstitutionPreimageV2 {
            semantics: scope.semantics,
            universe: scope.universe,
            formations,
            schemas: vec![q::RelationSchemaPreimageV2 {
                id: sid,
                roles: (1..=3)
                    .map(|i| q::RoleDeclarationPreimageV2 {
                        id: q::RoleLocalId::new(i),
                        target: target.clone(),
                        cardinality: one,
                        direct_dependencies: vec![],
                    })
                    .collect(),
                constraints: vec![],
                result_domain: target.clone(),
                direct_dependencies: vec![],
            }],
            capabilities: vec![q::CapabilityDeclarationPreimageV2 {
                id: capability,
                formation: q::FormationLocalId::new(4),
                direct_dependencies: vec![],
            }],
            operators: vec![q::OperatorPreimageV2 {
                id: op,
                modes: vec![
                    make_mode(mode, common_contract),
                    make_mode(checker, checker_contract),
                ],
                direct_dependencies: vec![],
            }],
            applications: vec![q::ApplicationDeclarationPreimageV2 {
                id: q::ApplicationLocalId::new(1),
                form: q::ApplicationFormPreimageV2 {
                    formation: q::FormationLocalId::new(5),
                    schema: sid,
                    operator: op,
                    eligible_modes: vec![mode, checker],
                    bindings: (1..=3)
                        .map(|i| q::RoleBindingPreimageV2 {
                            role: q::RoleLocalId::new(i),
                            occurrence: 0,
                            value: q::RoleBindingValuePreimageV2::Known(q::FormationLocalId::new(
                                i,
                            )),
                        })
                        .collect(),
                    context_requirements: vec![],
                    constraint_discharges: vec![],
                    result_domain: target,
                    direct_dependencies: vec![],
                    dependency_closure: dependencies,
                },
            }],
        },
        successor_grants: vec![],
        static_execution_grants: vec![],
        state_admission_grants: vec![],
        judgment_authority_grants: vec![],
    };
    let initial = atom(b"world-empty");
    let package = q::ProcessPackageV2 {
        claimed_snapshot: q::derive_program_snapshot_id(&snapshot).unwrap(),
        snapshot,
        initial_state_views: vec![q::InitialStateViewV2 {
            session: q::RuntimeSessionId::from_bytes(ident(702).0),
            canonical_state_snapshot: q::canonical_term_bytes(&initial)
                .unwrap()
                .into_boxed_slice(),
            payload: initial,
        }],
        records: vec![],
    };
    q::check_process_package(
        q::decode_process_package(&q::encode_process_package(&package).unwrap()).unwrap(),
    )
    .unwrap()
}
fn effect_process_expression(message: E) -> E {
    const MARKER: &[u8] = b"UNTRUSTED_LITERAL_MATERIALIZER_PAYLOAD_SLOT";
    let package = effect_process_package(MARKER);
    let snapshot = package.canonical_snapshot_preimage();
    let marker = [(MARKER.len() as u32).to_be_bytes().as_slice(), MARKER].concat();
    let locations = snapshot
        .windows(marker.len())
        .enumerate()
        .filter_map(|(i, b)| (b == marker).then_some(i))
        .collect::<Vec<_>>();
    assert_eq!(locations.len(), 1);
    let offset = locations[0];
    let_(
        "process-payload",
        message,
        let_(
            "process-snapshot",
            cat(vec![
                b(&snapshot[..offset]),
                call(LENGTH, vec![v("process-payload")]),
                v("process-payload"),
                b(&snapshot[offset + marker.len()..]),
            ]),
            cat(vec![
                b(b"CLPV\x02"),
                hash_expr(
                    "clause/program-snapshot/v1",
                    vec![
                        b(package.constitution().semantics().as_bytes()),
                        v("process-snapshot"),
                    ],
                ),
                v("process-snapshot"),
                b(&package.exact_bytes()[37 + snapshot.len()..]),
            ]),
        ),
    )
}
fn compiler_process_expression() -> E {
    use clause_package as q;
    const COMPILER: &[u8] = b"UNTRUSTED_LITERAL_MATERIALIZER_COMPILER_SLOT";
    const REQUEST: &[u8] = b"UNTRUSTED_LITERAL_MATERIALIZER_REQUEST_SLOT";
    let profiles = [
        core_contract_id().unwrap().as_bytes().as_slice(),
        physical_profile_id().unwrap().as_bytes().as_slice(),
    ]
    .concat();
    let base = effect_process_package(&profiles);
    let mut template = q::decode_process_package(base.exact_bytes())
        .unwrap()
        .candidate()
        .clone();
    let scope = q::TermScope {
        universe: template.snapshot.constitution.universe,
        semantics: template.snapshot.constitution.semantics,
    };
    for (id, payload) in [(1, COMPILER), (2, REQUEST)] {
        template
            .snapshot
            .constitution
            .formations
            .iter_mut()
            .find(|f| f.id == q::FormationLocalId::new(id))
            .unwrap()
            .term = q::Term::atom(
            scope,
            b"compiler-language/bytes".to_vec(),
            payload.to_vec(),
            q::EqualityContract::ExactOctetsV1,
        )
        .unwrap();
    }
    let remove = |d: &q::LocalSemanticDependencyV2| match d {
        q::LocalSemanticDependencyV2::Capability(_) => true,
        q::LocalSemanticDependencyV2::Formation(f) => *f == q::FormationLocalId::new(4),
        _ => false,
    };
    template
        .snapshot
        .constitution
        .formations
        .retain(|f| f.id != q::FormationLocalId::new(4));
    for formation in &mut template.snapshot.constitution.formations {
        formation.direct_dependencies.retain(|d| !remove(d));
    }
    template.snapshot.constitution.capabilities.clear();
    for mode in &mut template.snapshot.constitution.operators[0].modes {
        mode.contract.effect_intents.clear();
        mode.contract.capability_requirements.clear();
    }
    template.snapshot.constitution.applications[0]
        .form
        .dependency_closure
        .retain(|d| !remove(d));
    template.claimed_snapshot = q::derive_program_snapshot_id(&template.snapshot).unwrap();
    let package = q::check_process_package(
        q::decode_process_package(&q::encode_process_package(&template).unwrap()).unwrap(),
    )
    .unwrap();
    let snapshot = package.canonical_snapshot_preimage();
    let mut holes = vec![];
    for (marker, input) in [(COMPILER, 0), (REQUEST, 1)] {
        let marker = [(marker.len() as u32).to_be_bytes().as_slice(), marker].concat();
        let locations = snapshot
            .windows(marker.len())
            .enumerate()
            .filter_map(|(i, b)| (b == marker).then_some(i))
            .collect::<Vec<_>>();
        assert_eq!(locations.len(), 1);
        holes.push((
            locations[0],
            marker.len(),
            call(BLOB, vec![payload(field(v("request"), input))]),
        ));
    }
    assert_eq!(&snapshot[snapshot.len() - 16..], &[0; 16]);
    holes.push((snapshot.len() - 16, 4, payload(field(v("request"), 2))));
    holes.sort_by_key(|h| h.0);
    let mut parts = vec![];
    let mut offset = 0;
    for (start, len, value) in holes {
        parts.push(b(&snapshot[offset..start]));
        parts.push(value);
        offset = start + len;
    }
    parts.push(b(&snapshot[offset..]));
    let_(
        "compiler-process-snapshot",
        cat(parts),
        cat(vec![
            b(b"CLPV\x02"),
            hash_expr(
                "clause/program-snapshot/v1",
                vec![
                    b(scope.semantics.as_bytes()),
                    v("compiler-process-snapshot"),
                ],
            ),
            v("compiler-process-snapshot"),
            b(&package.exact_bytes()[37 + snapshot.len()..]),
        ]),
    )
}
fn hash_expr(domain_name: &str, components: Vec<E>) -> E {
    let mut parts = vec![b((domain_name.len() as u32).to_be_bytes()), b(domain_name)];
    for c in components {
        parts.push(cat(vec![b([0; 4]), call(LENGTH, vec![c.clone()])]));
        parts.push(c);
    }
    E::Sha(Box::new(cat(parts)))
}
fn new_id(kind: &[u8], occurrence: E, producer: u32) -> E {
    call(
        NEWID,
        vec![b(domain(kind).0), occurrence, b(ident(producer).0)],
    )
}
fn ok(ty: E, value: E, tree: E) -> E {
    rec(0x71, vec![ty, value, tree])
}
fn when_ok(result: E, yes: E) -> E {
    eq(payload(first(result.clone())), b([0x71]), yes, result)
}
fn diagnostic(code: &[u8], node: E, ctx: E) -> E {
    call(DIAG, vec![b(code), node, ctx])
}
fn inspect(node: E, env: E, ctx: E) -> E {
    call(CHECK, vec![node, env, ctx])
}
fn tree_node(kind: &[u8], node: E, ctx: E, fields: Vec<E>) -> E {
    rec(
        0x90,
        vec![
            tb(b(kind)),
            field(node, 0),
            field(ctx.clone(), 4),
            field(ctx, 1),
            ls(fields),
        ],
    )
}
fn language(p: &mut Vec<Definition>, successor: bool) {
    use KSort::{Bytes as B, Term as T};
    def(
        NEWID,
        &[("domain", B), ("occurrence", B), ("producer", B)],
        B,
        hash_expr(
            "clause/new-nominal/v1",
            vec![
                v("domain"),
                cat(vec![b(domain(b"change-occurrence").0), v("occurrence")]),
                cat(vec![b(domain(b"definition").0), v("producer")]),
                b([0; 8]),
            ],
        ),
        p,
    );
    let unresolved_doc = if successor {
        "Bind this name before using it."
    } else {
        "This name has no binding."
    };
    let unresolved_fix = if successor {
        "Introduce a binding in the containing scope."
    } else {
        "Declare this name."
    };
    let mut code_id = b(ident(304).0);
    let mut doc = b(b"This form is not available.");
    let mut fix = b(b"Use an available form.");
    for (index, code, message, repair) in [
        (0, b"unresolved".as_slice(), unresolved_doc, unresolved_fix),
        (
            1,
            b"type".as_slice(),
            "This expression needs Text.",
            "Supply a Text value.",
        ),
        (
            2,
            b"phase".as_slice(),
            "This transformation is unavailable here.",
            "Use it where its input is available.",
        ),
        (
            3,
            b"capability".as_slice(),
            "This operation needs access to the mailbox.",
            "Supply the mailbox capability.",
        ),
    ] {
        code_id = eq(v("code"), b(code), b(ident(300 + index).0), code_id);
        doc = eq(v("code"), b(code), b(message), doc);
        fix = eq(v("code"), b(code), b(repair), fix);
    }
    let diagnostic_record = rec(
        0x91,
        vec![
            tb(new_id(
                b"diagnostic-occurrence",
                payload(field(v("node"), 0)),
                DIAG,
            )),
            rec(
                0x04,
                vec![t(idterm(domain(b"referent"))), at(b(ID), code_id, b(EQ))],
            ),
            tb(b(b"error")),
            field(v("ctx"), 4),
            ls(vec![]),
            ls(vec![]),
            tb(doc),
            ls(vec![rec(0x96, vec![field(v("ctx"), 4), tb(fix)])]),
        ],
    );
    let diag = rec(0x15, vec![ls(vec![diagnostic_record])]);
    def(DIAG, &[("code", B), ("node", T), ("ctx", T)], T, diag, p);
    def(
        FIND,
        &[("name", B), ("env", T)],
        T,
        ct(
            v("env"),
            ["m", "entry", "tail"],
            t(nil()),
            eq(
                payload(field(v("entry"), 1)),
                v("name"),
                v("entry"),
                call(FIND, vec![v("name"), v("tail")]),
            ),
        ),
        p,
    );
    let binding = let_(
        "rhs",
        inspect(field(v("node"), 2), v("env"), v("ctx")),
        when_ok(
            v("rhs"),
            let_(
                "binder",
                tb(new_id(b"binder", payload(field(v("node"), 0)), BIND)),
                let_(
                    "scope",
                    tb(new_id(b"scope", payload(field(v("node"), 0)), BIND)),
                    let_(
                        "entry",
                        rec(
                            0x72,
                            vec![
                                v("binder"),
                                field(v("node"), 1),
                                field(v("rhs"), 0),
                                field(v("rhs"), 1),
                                field(v("ctx"), 1),
                                v("scope"),
                                field(v("ctx"), 4),
                            ],
                        ),
                        let_(
                            "body",
                            inspect(
                                field(v("node"), 3),
                                tr(t(tag(1)), v("entry"), v("env")),
                                v("ctx"),
                            ),
                            when_ok(
                                v("body"),
                                ok(
                                    field(v("body"), 0),
                                    field(v("body"), 1),
                                    tree_node(
                                        b"binding",
                                        v("node"),
                                        v("ctx"),
                                        vec![v("entry"), field(v("rhs"), 2), field(v("body"), 2)],
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    let mut binding_reading = eq(
        payload(first(v("node"))),
        b(b"bind"),
        binding.clone(),
        t(nil()),
    );
    if successor {
        binding_reading = eq(
            payload(first(v("node"))),
            b(b"with"),
            binding,
            binding_reading,
        );
    }
    def(
        BIND,
        &[("node", T), ("env", T), ("ctx", T)],
        T,
        binding_reading,
        p,
    );
    let effect = let_(
        "payload",
        inspect(field(v("node"), 1), v("env"), v("ctx")),
        when_ok(
            v("payload"),
            eq(
                payload(field(v("payload"), 0)),
                b(b"Text"),
                eq(
                    payload(field(v("ctx"), 2)),
                    b(b"mailbox"),
                    ok(
                        t(bytes(b"Text")),
                        field(v("payload"), 1),
                        tree_node(
                            b"effect",
                            v("node"),
                            v("ctx"),
                            vec![
                                rec(
                                    0x9a,
                                    vec![
                                        tb(b(b"notify")),
                                        tb(b(b"mailbox")),
                                        tb(b(b"outbox")),
                                        field(v("payload"), 1),
                                        field(v("ctx"), 4),
                                        tb(new_id(
                                            b"effect-intent",
                                            payload(field(v("node"), 0)),
                                            EFFECT,
                                        )),
                                        tb(b(b"admit-intent;authorize-activation;attempt-once")),
                                        tb(effect_process_expression(payload(field(
                                            v("payload"),
                                            1,
                                        )))),
                                    ],
                                ),
                                field(v("payload"), 2),
                            ],
                        ),
                    ),
                    diagnostic(b"capability", v("node"), v("ctx")),
                ),
                diagnostic(b"type", v("node"), v("ctx")),
            ),
        ),
    );
    def(
        EFFECT,
        &[("node", T), ("env", T), ("ctx", T)],
        T,
        if successor {
            eq(payload(first(v("node"))), b(b"notify"), effect, t(nil()))
        } else {
            t(nil())
        },
        p,
    );
    let derived_origin = rec(
        0x82,
        vec![
            field(v("node"), 0),
            field(v("ctx"), 4),
            t(number(0)),
            field(v("ctx"), 1),
        ],
    );
    let macro_body = let_(
        "operand",
        inspect(field(v("node"), 1), v("env"), v("ctx")),
        when_ok(
            v("operand"),
            eq(
                payload(field(v("operand"), 0)),
                b(b"Text"),
                let_(
                    "body",
                    inspect(field(v("node"), 2), v("env"), v("ctx")),
                    when_ok(
                        v("body"),
                        eq(
                            payload(field(v("body"), 0)),
                            b(b"Text"),
                            let_(
                                "binder",
                                tb(new_id(b"binder", payload(field(v("node"), 0)), MACRO)),
                                let_(
                                    "scope",
                                    tb(new_id(b"scope", payload(field(v("node"), 0)), MACRO)),
                                    let_(
                                        "derived",
                                        derived_origin.clone(),
                                        let_(
                                            "origin",
                                            tb(hash_expr(
                                                "clause/origin/v1",
                                                vec![call(ENCTERM, vec![v("derived")])],
                                            )),
                                            ok(
                                                t(bytes(b"Pair<Text>")),
                                                tr(
                                                    field(v("operand"), 1),
                                                    field(v("body"), 1),
                                                    t(nil()),
                                                ),
                                                tree_node(
                                                    b"typed-macro",
                                                    v("node"),
                                                    v("ctx"),
                                                    vec![
                                                        rec(
                                                            0x72,
                                                            vec![
                                                                v("binder"),
                                                                tb(b(b"tmp")),
                                                                t(bytes(b"Text")),
                                                                field(v("operand"), 1),
                                                                field(v("ctx"), 1),
                                                                v("scope"),
                                                                v("origin"),
                                                            ],
                                                        ),
                                                        rec(
                                                            0x73,
                                                            vec![
                                                                tb(new_id(
                                                                    b"use",
                                                                    payload(field(v("node"), 0)),
                                                                    MACRO,
                                                                )),
                                                                v("binder"),
                                                                field(v("ctx"), 1),
                                                                v("origin"),
                                                            ],
                                                        ),
                                                        v("derived"),
                                                        v("origin"),
                                                        field(v("operand"), 2),
                                                        field(v("body"), 2),
                                                    ],
                                                ),
                                            ),
                                        ),
                                    ),
                                ),
                            ),
                            diagnostic(b"type", v("node"), v("ctx")),
                        ),
                    ),
                ),
                diagnostic(b"type", v("node"), v("ctx")),
            ),
        ),
    );
    def(
        MACRO,
        &[("node", T), ("env", T), ("ctx", T)],
        T,
        if successor {
            eq(
                payload(first(v("node"))),
                b(b"share"),
                eq(
                    payload(field(v("ctx"), 1)),
                    b(b"runtime"),
                    macro_body,
                    diagnostic(b"phase", v("node"), v("ctx")),
                ),
                t(nil()),
            )
        } else {
            t(nil())
        },
        p,
    );
    let fallback = let_(
        "binding",
        call(BIND, vec![v("node"), v("env"), v("ctx")]),
        eq(
            payload(first(v("binding"))),
            b([0]),
            let_(
                "effect",
                call(EFFECT, vec![v("node"), v("env"), v("ctx")]),
                eq(
                    payload(first(v("effect"))),
                    b([0]),
                    let_(
                        "macro",
                        call(MACRO, vec![v("node"), v("env"), v("ctx")]),
                        eq(
                            payload(first(v("macro"))),
                            b([0]),
                            diagnostic(b"syntax", v("node"), v("ctx")),
                            v("macro"),
                        ),
                    ),
                    v("effect"),
                ),
            ),
            v("binding"),
        ),
    );
    let literal = ok(
        t(bytes(b"Text")),
        field(v("node"), 1),
        tree_node(b"literal", v("node"), v("ctx"), vec![]),
    );
    let use_ = let_(
        "entry",
        call(FIND, vec![payload(field(v("node"), 1)), v("env")]),
        eq(
            payload(first(v("entry"))),
            b([0]),
            diagnostic(b"unresolved", v("node"), v("ctx")),
            eq(
                payload(field(v("entry"), 4)),
                payload(field(v("ctx"), 1)),
                ok(
                    field(v("entry"), 2),
                    field(v("entry"), 3),
                    tree_node(
                        b"use",
                        v("node"),
                        v("ctx"),
                        vec![rec(
                            0x73,
                            vec![
                                field(v("node"), 0),
                                field(v("entry"), 0),
                                field(v("entry"), 4),
                                field(v("ctx"), 4),
                            ],
                        )],
                    ),
                ),
                diagnostic(b"phase", v("node"), v("ctx")),
            ),
        ),
    );
    let closure = ok(
        t(bytes(b"Closure")),
        rec(0x80, vec![field(v("node"), 1), v("env")]),
        tree_node(b"closure", v("node"), v("ctx"), vec![v("env")]),
    );
    let apply = let_(
        "function",
        inspect(field(v("node"), 1), v("env"), v("ctx")),
        when_ok(
            v("function"),
            eq(
                payload(field(v("function"), 0)),
                b(b"Closure"),
                let_(
                    "closure",
                    field(v("function"), 1),
                    let_(
                        "result",
                        inspect(field(v("closure"), 0), field(v("closure"), 1), v("ctx")),
                        when_ok(
                            v("result"),
                            ok(
                                field(v("result"), 0),
                                field(v("result"), 1),
                                tree_node(
                                    b"application",
                                    v("node"),
                                    v("ctx"),
                                    vec![field(v("function"), 2), field(v("result"), 2)],
                                ),
                            ),
                        ),
                    ),
                ),
                diagnostic(b"type", v("node"), v("ctx")),
            ),
        ),
    );
    let pair = let_(
        "left",
        inspect(field(v("node"), 1), v("env"), v("ctx")),
        when_ok(
            v("left"),
            let_(
                "right",
                inspect(field(v("node"), 2), v("env"), v("ctx")),
                when_ok(
                    v("right"),
                    ok(
                        t(bytes(b"Pair")),
                        tr(field(v("left"), 1), field(v("right"), 1), t(nil())),
                        tree_node(
                            b"pair",
                            v("node"),
                            v("ctx"),
                            vec![field(v("left"), 2), field(v("right"), 2)],
                        ),
                    ),
                ),
            ),
        ),
    );
    let mut reading = fallback;
    for (name, body) in [
        (b"text".as_slice(), literal),
        (b"use".as_slice(), use_),
        (b"fn".as_slice(), closure),
        (b"apply".as_slice(), apply),
        (b"pair".as_slice(), pair),
    ] {
        reading = eq(payload(first(v("node"))), b(name), body, reading);
    }
    def(CHECK, &[("node", T), ("env", T), ("ctx", T)], T, reading, p);
    let analyze = let_(
        "source",
        payload(field(v("request"), 0)),
        let_(
            "artifact",
            tb(hash_expr("clause/source-artifact/v1", vec![v("source")])),
            let_(
                "origin-node",
                rec(
                    0x81,
                    vec![
                        v("artifact"),
                        t(number(0)),
                        at(
                            b(U64),
                            cat(vec![b([0; 4]), call(LENGTH, vec![v("source")])]),
                            b(EQ),
                        ),
                    ],
                ),
                let_(
                    "origin",
                    tb(hash_expr(
                        "clause/origin/v1",
                        vec![call(ENCTERM, vec![v("origin-node")])],
                    )),
                    let_(
                        "ctx",
                        rec(
                            0x70,
                            vec![
                                field(v("request"), 0),
                                field(v("request"), 1),
                                field(v("request"), 2),
                                field(v("request"), 3),
                                v("origin"),
                                v("origin-node"),
                            ],
                        ),
                        let_(
                            "parsed",
                            call(READTERM, vec![v("source")]),
                            eq(
                                payload(third(v("parsed"))),
                                b(b"ok"),
                                eq(
                                    payload(second(v("parsed"))),
                                    b([]),
                                    rec(
                                        0x95,
                                        vec![
                                            field(v("request"), 0),
                                            rec(
                                                0x83,
                                                vec![
                                                    v("artifact"),
                                                    t(number(0)),
                                                    at(
                                                        b(U64),
                                                        cat(vec![
                                                            b([0; 4]),
                                                            call(LENGTH, vec![v("source")]),
                                                        ]),
                                                        b(EQ),
                                                    ),
                                                    v("origin"),
                                                ],
                                            ),
                                            v("origin-node"),
                                            inspect(first(v("parsed")), t(nil()), v("ctx")),
                                        ],
                                    ),
                                    diagnostic(
                                        b"syntax",
                                        rec(0, vec![field(v("request"), 3)]),
                                        v("ctx"),
                                    ),
                                ),
                                diagnostic(
                                    b"syntax",
                                    rec(0, vec![field(v("request"), 3)]),
                                    v("ctx"),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    def(ANALYZE, &[("request", T)], T, analyze, p);
}

fn parts(t: &Term) -> (&Term, &Term, &Term) {
    let Term::Triple(a, b, c) = t else {
        panic!("expected triple")
    };
    (a, b, c)
}
fn data(t: &Term) -> &[u8] {
    let Term::Atom {
        canonical_payload, ..
    } = t
    else {
        panic!("expected atom")
    };
    canonical_payload
}
fn at_field(t: &Term, index: usize) -> &Term {
    let mut fields = parts(t).1;
    for _ in 0..index {
        fields = parts(fields).2;
    }
    parts(fields).1
}
fn syntax(name: &[u8], occurrence: u32, fields: Vec<Term>) -> Term {
    let mut all = vec![bytes(&ident(occurrence).0)];
    all.extend(fields);
    triple(bytes(name), list(all), nil())
}
fn text_s(occ: u32, value: &[u8]) -> Term {
    syntax(b"text", occ, vec![bytes(value)])
}
fn use_s(occ: u32, name: &[u8]) -> Term {
    syntax(b"use", occ, vec![bytes(name)])
}
fn binding_s(kind: &[u8], occ: u32, name: &[u8], rhs: Term, body: Term) -> Term {
    syntax(kind, occ, vec![bytes(name), rhs, body])
}
fn closure_spec(kind: &[u8], x: &[u8], f: &[u8]) -> Term {
    binding_s(
        b"bind",
        500,
        x,
        text_s(501, b"outer"),
        binding_s(
            b"bind",
            502,
            f,
            syntax(b"fn", 503, vec![use_s(504, x)]),
            binding_s(
                kind,
                505,
                x,
                text_s(506, b"inner"),
                syntax(
                    b"pair",
                    507,
                    vec![use_s(508, x), syntax(b"apply", 509, vec![use_s(510, f)])],
                ),
            ),
        ),
    )
}
fn macro_spec() -> Term {
    binding_s(
        b"bind",
        520,
        b"tmp",
        text_s(521, b"caller"),
        syntax(
            b"share",
            522,
            vec![text_s(523, b"generated"), use_s(524, b"tmp")],
        ),
    )
}
fn analyze_request(source: &[u8], phase: &[u8], capability: &[u8]) -> Term {
    record(
        0x70,
        vec![
            bytes(source),
            bytes(phase),
            bytes(capability),
            bytes(&ident(600).0),
        ],
    )
}
fn write_eval(root: &Path, name: &str, evaluation: &clause_substrate::evaluator::Evaluation) {
    let mut bytes_ = vec![];
    match &evaluation.value {
        KValue::Term(t) => {
            bytes_.push(1);
            bytes_.extend(encode_canonical_term(t).unwrap())
        }
        KValue::Bytes(b) => {
            bytes_.push(0);
            bytes_.extend((b.len() as u32).to_be_bytes());
            bytes_.extend(b)
        }
    }
    fs::write(root.join(format!("{name}.value")), bytes_).unwrap();
    fs::write(
        root.join(format!("{name}.value.observations")),
        encode_canonical_term(&evaluation.observations.try_to_term().unwrap()).unwrap(),
    )
    .unwrap();
    fs::write(
        root.join(format!("{name}.value.fuel")),
        format!("{}\n", evaluation.remaining_fuel),
    )
    .unwrap();
}
fn analyzed_result(e: &clause_substrate::evaluator::Evaluation) -> &Term {
    let KValue::Term(t) = &e.value else {
        panic!("analyzer result sort")
    };
    assert_eq!(data(parts(t).0), [0x95]);
    at_field(t, 3)
}
fn assert_value(e: &clause_substrate::evaluator::Evaluation, expected: &Term) {
    let result = analyzed_result(e);
    assert_eq!(data(parts(result).0), [0x71], "checked expression");
    assert_eq!(at_field(result, 1), expected);
}
fn records<'a>(term: &'a Term, tag_: u8, out: &mut Vec<&'a Term>) {
    if let Term::Triple(a, b, c) = term {
        if matches!(&**a,Term::Atom{kind,canonical_payload,..}if kind==TAG&&canonical_payload==&[tag_])
        {
            out.push(term);
        }
        records(a, tag_, out);
        records(b, tag_, out);
        records(c, tag_, out);
    }
}
fn language_specs(root: &Path, p0: &[Definition], p1: &[Definition]) {
    let e0 = Evaluator::new(p0).unwrap();
    let e1 = Evaluator::new(p1).unwrap();
    let specimens = vec![
        (
            "old-binding",
            closure_spec(b"bind", b"x", b"f"),
            b"runtime".as_slice(),
            b"mailbox".as_slice(),
        ),
        (
            "new-binding",
            closure_spec(b"with", b"x", b"f"),
            b"runtime",
            b"mailbox",
        ),
        (
            "alpha-renamed",
            closure_spec(b"with", b"z", b"g"),
            b"runtime",
            b"mailbox",
        ),
        ("typed-macro", macro_spec(), b"runtime", b"mailbox"),
        ("macro-phase", macro_spec(), b"compile", b"mailbox"),
        (
            "macro-type",
            syntax(
                b"share",
                550,
                vec![
                    syntax(b"fn", 551, vec![text_s(552, b"closure")]),
                    text_s(553, b"body"),
                ],
            ),
            b"runtime",
            b"mailbox",
        ),
        (
            "macro-without-capability",
            syntax(
                b"share",
                554,
                vec![
                    syntax(b"notify", 555, vec![text_s(556, b"message")]),
                    text_s(557, b"body"),
                ],
            ),
            b"runtime",
            b"",
        ),
        (
            "effect",
            syntax(b"notify", 530, vec![text_s(531, b"message")]),
            b"runtime",
            b"mailbox",
        ),
        (
            "effect-without-capability",
            syntax(b"notify", 530, vec![text_s(531, b"message")]),
            b"runtime",
            b"",
        ),
        (
            "macro-effect",
            syntax(
                b"share",
                532,
                vec![
                    syntax(b"notify", 533, vec![text_s(534, b"one message")]),
                    text_s(535, b"body"),
                ],
            ),
            b"runtime",
            b"mailbox",
        ),
        ("unresolved", use_s(540, b"missing"), b"runtime", b"mailbox"),
    ];
    let mut kept = std::collections::BTreeMap::new();
    for (name, node, phase, capability) in specimens {
        let mut source = vec![];
        textual(&node, &mut source);
        let req = analyze_request(&source, phase, capability);
        fs::write(root.join(format!("{name}.source")), &source).unwrap();
        fs::write(
            root.join(format!("{name}.request.term")),
            encode_canonical_term(&req).unwrap(),
        )
        .unwrap();
        let zero = e0
            .invoke_entrypoint(
                ident(ANALYZE),
                &[KValue::Term(clone_term(&req))],
                10_000_000,
            )
            .unwrap();
        let one = e1
            .invoke_entrypoint(ident(ANALYZE), &[KValue::Term(req)], 10_000_000)
            .unwrap();
        write_eval(root, &format!("{name}.compiler0"), &zero);
        write_eval(root, &format!("{name}.compiler1"), &one);
        match name {
            "old-binding" => {
                assert_value(&zero, &triple(bytes(b"inner"), bytes(b"outer"), nil()));
                assert_value(&one, &triple(bytes(b"inner"), bytes(b"outer"), nil()));
                assert_eq!(zero.value, one.value);
            }
            "new-binding" | "alpha-renamed" => {
                assert_eq!(data(parts(analyzed_result(&zero)).0), [0x15]);
                assert_value(&one, &triple(bytes(b"inner"), bytes(b"outer"), nil()));
            }
            "typed-macro" => {
                assert_eq!(data(parts(analyzed_result(&zero)).0), [0x15]);
                assert_value(&one, &triple(bytes(b"generated"), bytes(b"caller"), nil()));
                let mut binders = vec![];
                records(analyzed_result(&one), 0x72, &mut binders);
                let ids = binders
                    .iter()
                    .map(|x| data(at_field(x, 0)))
                    .collect::<std::collections::BTreeSet<_>>();
                assert_eq!(ids.len(), 2, "macro binder cannot capture caller binder");
            }
            "macro-phase"
            | "macro-type"
            | "macro-without-capability"
            | "effect-without-capability" => {
                assert_eq!(data(parts(analyzed_result(&one)).0), [0x15]);
            }
            "effect" => {
                assert_eq!(data(parts(analyzed_result(&zero)).0), [0x15]);
                assert_value(&one, &bytes(b"message"));
                let mut effects = vec![];
                records(analyzed_result(&one), 0x9a, &mut effects);
                assert_eq!(effects.len(), 1);
                let emitted = data(at_field(effects[0], 7));
                let checked = clause_package::check_process_package(
                    clause_package::decode_process_package(emitted).unwrap(),
                )
                .unwrap();
                assert_eq!(
                    checked.exact_bytes(),
                    effect_process_package(b"message").exact_bytes()
                );
                fs::write(root.join("effect.process.clpv"), emitted).unwrap();
            }
            "macro-effect" => {
                assert_value(&one, &triple(bytes(b"one message"), bytes(b"body"), nil()));
                let mut effects = vec![];
                records(analyzed_result(&one), 0x9a, &mut effects);
                assert_eq!(effects.len(), 1, "typed macro preserves one effect intent");
            }
            "unresolved" => {
                let mut a = vec![];
                let mut b = vec![];
                records(analyzed_result(&zero), 0x91, &mut a);
                records(analyzed_result(&one), 0x91, &mut b);
                assert_eq!(a.len(), 1);
                assert_eq!(b.len(), 1);
                for field in 0..6 {
                    assert_eq!(
                        at_field(a[0], field),
                        at_field(b[0], field),
                        "diagnostic identity, code, severity, origin and rejection are stable"
                    );
                }
                assert_ne!(at_field(a[0], 6), at_field(b[0], 6));
                assert_ne!(at_field(a[0], 7), at_field(b[0], 7));
            }
            _ => unreachable!(),
        }
        println!("{name}: package-owned observation checked");
        kept.insert(name, one);
    }
    let identities = |e: &clause_substrate::evaluator::Evaluation| {
        let mut rows = vec![];
        records(analyzed_result(e), 0x73, &mut rows);
        rows.into_iter()
            .map(|r| (data(at_field(r, 0)).to_vec(), data(at_field(r, 1)).to_vec()))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        identities(&kept["new-binding"]),
        identities(&kept["alpha-renamed"]),
        "alpha-renaming retains each use-to-binder edge"
    );
}
