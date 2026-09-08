use clause_package::*;

#[test]
fn incremental_scalar_analysis_matches_full_check_and_rejects_stale_or_invalid_edits() {
    let encounter = include_bytes!("../../../test-vectors/authoring/live-encounter.clause");
    let collection = [encounter.as_slice(), b"\n", include_bytes!("../../../test-vectors/authoring/created-burn-extension.clause")].concat();
    let imported = [b"import \"shared.clause\"\n".as_slice(), encounter.as_slice(), b"\nexport local-label(): Text\n  shared-label()\n"].concat();
    let imports = CanonicalSourceImportsV1::from([("shared.clause".into(), b"export shared-label(): Text\n  \"retained\"\n".to_vec())]);
    for (source, imports) in [(encounter.as_slice(), CanonicalSourceImportsV1::new()), (collection.as_slice(), CanonicalSourceImportsV1::new()), (imported.as_slice(), imports)] {
        let cst = read_canonical_source_with_imports_v1(source, &imports).unwrap();
        let plan = plan_independent_canonical_source_allocations_v1(&cst, ProgramChangeOccurrenceId::from_bytes([3;32])).unwrap();
        let context = CanonicalSourceContextV1 { universe: UniverseId::from_bytes([1;32]), semantics: ClauseSemanticsId::from_bytes([2;32]) };
        let mut checked = CheckedCanonicalSourceAnalysisV1::new(cst, plan, context).unwrap();
        for (index, (before, after)) in [("0.0 - ?damage", "0.0 - (?damage * 2.0)"), ("0.0 - (?damage * 2.0)", "0.0 - ?damage")].into_iter().enumerate() {
            let offered = canonical_scalar_effects_v1(checked.source(), checked.plan()).unwrap();
            assert_eq!(checked.scalar_effects().unwrap(), offered);
            let selected = offered.into_iter().find(|effect| effect.expression == before.as_bytes()).unwrap();
            let root = ProgramChangeOccurrenceId::from_bytes([4 + index as u8;32]);
            let invalid = replace_canonical_scalar_effect_v1(checked.source(), checked.plan(), &selected, b"true", root).unwrap();
            assert!(checked.advance(&invalid).is_err());
            let full_edit = replace_canonical_scalar_effect_v1(checked.source(), checked.plan(), &selected, after.as_bytes(), root).unwrap();
            let edit = checked.replace_scalar_effect(selected.handler, selected.effect, &selected.field_path, after.as_bytes(), root).unwrap();
            assert_eq!(edit.source().exact_source(), full_edit.source().exact_source());
            assert_eq!(edit.plan(), full_edit.plan());
            assert_eq!(edit.retained(), full_edit.retained());
            let next = checked.advance(&edit).unwrap();
            let mut retained_count = 0;
            for handler in &next.package().executable_handlers {
                for index in 0..handler.rules.len() {
                    assert!(next.retained_rule(root, handler.id, index).is_none());
                    let retained = next.retained_rule(checked.plan().root(), handler.id, index);
                    if handler.id == edit.formation(selected.handler).unwrap() {
                        assert!(retained.is_none());
                    } else {
                        let (old_handler, old_index) = retained.unwrap();
                        assert_eq!(edit.formation(old_handler).unwrap(), handler.id);
                        assert!(old_index < checked.package().executable_handlers.iter().find(|handler| handler.id == old_handler).unwrap().rules.len());
                        retained_count += 1;
                    }
                }
            }
            assert!(retained_count > 0);
            let full_source = read_canonical_source_with_imports_v1(edit.source().exact_source(), &imports).unwrap();
            let full_plan = plan_independent_canonical_source_allocations_v1(&full_source, root).unwrap();
            assert_eq!(edit.plan(), &full_plan);
            assert_eq!(next.scalar_effects().unwrap(), canonical_scalar_effects_v1(&full_source, &full_plan).unwrap());
            assert_eq!(next.source().vocabularies(), full_source.vocabularies());
            assert_eq!(next.source().subject_focuses(), full_source.subject_focuses());
            assert_eq!(next.source().denotations(), full_source.denotations());
            assert_eq!(next.source().applications(), full_source.applications());
            let full = elaborate_canonical_source_package_v1(&full_source, context, &full_plan).unwrap();
            let incremental = next.package();
            let wire_checked = check_process_package(decode_process_package(incremental.checked_package.exact_bytes()).unwrap()).unwrap();
            assert_eq!(wire_checked.id(), incremental.checked_package.id());
            assert_eq!(wire_checked.constitution(), incremental.checked_package.constitution());
            assert_eq!(next.source().imports(), &imports);
            assert_eq!(incremental.callables, full.callables);
            assert_eq!(incremental.checked_package.exact_bytes(), full.checked_package.exact_bytes());
            assert_eq!(incremental.executable_handlers.len(), full.executable_handlers.len());
            for (i, (actual, expected)) in incremental.executable_handlers.iter().zip(&full.executable_handlers).enumerate() {
                if actual != expected {
                    let a = format!("{actual:#?}"); let b = format!("{expected:#?}");
                    let differing = a.lines().zip(b.lines()).enumerate().find(|(_, (a,b))| a != b);
                    let line = differing.map(|(line, _)| line).unwrap_or(0);
                    panic!("handler {i} {:?} differs; actual {:?}; expected {:?}", String::from_utf8_lossy(&actual.designation), a.lines().skip(line.saturating_sub(8)).take(18).collect::<Vec<_>>(), b.lines().skip(line.saturating_sub(8)).take(18).collect::<Vec<_>>());
                }
            }
            assert_eq!(incremental.state_cells, full.state_cells);
            assert_eq!(incremental.relational_projection, full.relational_projection);
            assert_eq!(incremental.emissions, full.emissions);
            assert_eq!(incremental.keyboard_bindings, full.keyboard_bindings);
            assert_eq!(incremental.scalar_input_bindings, full.scalar_input_bindings);
            assert_eq!(incremental.referent_input_bindings, full.referent_input_bindings);
            assert_eq!(incremental.input_handler, full.input_handler);
            assert_eq!(incremental.scalar_handlers, full.scalar_handlers);
            assert!(next.advance(&edit).is_err());
            assert!(next.replace_scalar_effect(selected.handler, selected.effect, &selected.field_path, after.as_bytes(), root).is_err());
            checked = next;
        }
    }
}
