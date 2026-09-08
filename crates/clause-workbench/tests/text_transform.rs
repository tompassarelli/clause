use clause_runtime::ExecutableValueV1 as V;
use clause_workbench::ResidentSourceWorkbenchV1;
use std::{fs, path::PathBuf, process::Command};

#[test]
fn text_transform_family_and_mark_fold_agree_in_native_and_bun() {
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/text-transform");
    fs::create_dir_all(&output).unwrap();
    let family = ResidentSourceWorkbenchV1::open(include_bytes!("../../../test-vectors/authoring/text-transform/family.clause")).unwrap();
    let minimal = ResidentSourceWorkbenchV1::open(include_bytes!("../../../test-vectors/authoring/text-transform/lowercase.clause")).unwrap();
    let marks = ResidentSourceWorkbenchV1::open(include_bytes!("../../../test-vectors/authoring/text-transform/mark-fold.clause")).unwrap();
    for (name, opened) in [("family", &family), ("minimal", &minimal), ("marks", &marks)] {
        let generated = clause_package::lower_javascript_v1(&opened.checked_source_package().unwrap()).unwrap();
        fs::write(output.join(format!("{name}.mjs")), generated.module).unwrap();
    }
    let inputs = ["", "  \t\n", "  First\tsecond  third \n", "\u{85}One\u{85}Two\u{85}", "\u{feff}One Two\u{feff}", "\u{a0}ÉCOLE\u{2003}ΟΣ İ\u{3000}", "ΟΣ ΣΟΣ ΟΣΑ İ ẞ 𐐀", "first\r\nrest\t "];
    let mut expected = String::new();
    for input in inputs {
        for name in [b"trimmed".as_slice(), b"first", b"remaining", b"lower"] {
            let value = family.invoke_callable(name, &[V::text(input).unwrap()]).unwrap();
            expected.push_str(value.as_text().unwrap());
            expected.push('\0');
        }
        let value = minimal.invoke_callable(b"normalize-letter", &[V::text(input).unwrap()]).unwrap();
        assert_eq!(value, family.invoke_callable(b"lower", &[V::text(input).unwrap()]).unwrap());
    }
    let mark = |letter: &str, window: f64| marks.invoke_callable(b"mark", &[V::text(letter).unwrap(), V::Number(window.to_bits()), V::text("app").unwrap(), V::text("title").unwrap()]).unwrap();
    let first = mark("A", 1.0);
    let second = mark("B", 2.0);
    let replacement = mark("a", 3.0);
    assert_eq!(marks.invoke_callable(b"fold-marks", &[V::Sequence(vec![first, second.clone(), replacement.clone()])]).unwrap(), V::Sequence(vec![replacement, second]));
    fs::write(output.join("inputs.txt"), inputs.join("\0")).unwrap();
    fs::write(output.join("check.mjs"), r#"import * as f from './family.mjs';
import {"normalize-letter" as normalizeLetter} from './minimal.mjs';
import {mark,"fold-marks" as foldMarks} from './marks.mjs';
const inputs=(await Bun.file('inputs.txt').text()).split('\0');
for(const input of inputs){for(const name of ['trimmed','first','remaining','lower'])process.stdout.write(f[name](input)+'\0');if(normalizeLetter(input)!==f.lower(input))throw Error('minimal lowercase mismatch');}
const a=mark('A',1,'app','title'),b=mark('B',2,'app','title'),replacement=mark('a',3,'app','title');
if(JSON.stringify(foldMarks([a,b,replacement]))!==JSON.stringify([replacement,b]))throw Error('mark ordering or replacement mismatch');
"#).unwrap();
    let bun = std::env::var("BUN").unwrap_or_else(|_| "bun".into());
    let result = Command::new(bun).arg("check.mjs").current_dir(&output).output().unwrap();
    fs::write(output.join("bun.stderr"), &result.stderr).unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    assert_eq!(result.stdout, expected.as_bytes());
}
