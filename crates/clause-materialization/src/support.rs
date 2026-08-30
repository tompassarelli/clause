//! Occurrence-exact visible support indexes.

use std::collections::{BTreeMap, BTreeSet};

use crate::model::{OpaqueRef, OpaqueValue, SupportRecord};
use crate::work::WorkCounters;

pub(crate) type AnchorKey = Vec<(OpaqueRef, OpaqueValue)>;

/// A borrowed allocation-free observation of one materialized view.
#[derive(Clone, Copy)]
pub struct MaterializedView<'a> {
    records: &'a BTreeMap<OpaqueRef, SupportRecord>,
    visible: &'a VisibleSupportStore,
}

impl<'a> MaterializedView<'a> {
    pub fn supports(&self) -> impl Iterator<Item = &'a SupportRecord> + 'a {
        self.visible
            .visible
            .iter()
            .filter_map(|support_ref| self.records.get(support_ref))
    }

    #[must_use]
    pub fn outputs(&self) -> impl ExactSizeIterator<Item = MaterializedOutput<'_>> + '_ {
        self.visible
            .by_output
            .iter()
            .map(move |(output, support_refs)| MaterializedOutput {
                output,
                support_refs,
                records: self.records,
            })
    }

    pub fn supports_for(&self, output: &OpaqueRef) -> impl Iterator<Item = &'a SupportRecord> + 'a {
        self.visible
            .by_output
            .get(output)
            .into_iter()
            .flat_map(|support_refs| {
                support_refs
                    .iter()
                    .filter_map(|support_ref| self.records.get(support_ref))
            })
    }

    #[must_use]
    pub fn premise_multiplicity(
        &self,
        premise_occurrence_ref: &OpaqueRef,
        support_occurrence_ref: &OpaqueRef,
    ) -> u64 {
        self.visible
            .by_premise
            .get(premise_occurrence_ref)
            .and_then(|supports| supports.get(support_occurrence_ref))
            .copied()
            .unwrap_or(0)
    }
}

/// One borrowed output and every distinct support occurrence for it.
#[derive(Clone, Copy)]
pub struct MaterializedOutput<'a> {
    output: &'a OpaqueRef,
    support_refs: &'a BTreeSet<OpaqueRef>,
    records: &'a BTreeMap<OpaqueRef, SupportRecord>,
}

impl<'a> MaterializedOutput<'a> {
    #[must_use]
    pub fn output(&self) -> &'a OpaqueRef {
        self.output
    }

    pub fn supports(&self) -> impl Iterator<Item = &'a SupportRecord> + 'a {
        self.support_refs
            .iter()
            .filter_map(|support_ref| self.records.get(support_ref))
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct VisibleSupportStore {
    visible: BTreeSet<OpaqueRef>,
    by_output: BTreeMap<OpaqueRef, BTreeSet<OpaqueRef>>,
    by_premise: BTreeMap<OpaqueRef, BTreeMap<OpaqueRef, u64>>,
    by_anchor: BTreeMap<AnchorKey, BTreeSet<OpaqueRef>>,
    anchor_by_support: BTreeMap<OpaqueRef, AnchorKey>,
}

impl VisibleSupportStore {
    pub(crate) fn view<'a>(
        &'a self,
        records: &'a BTreeMap<OpaqueRef, SupportRecord>,
    ) -> MaterializedView<'a> {
        MaterializedView {
            records,
            visible: self,
        }
    }

    pub(crate) fn attach_scan(&mut self, record: &SupportRecord, counters: &mut WorkCounters) {
        if !self.visible.insert(record.support_occurrence_ref.clone()) {
            return;
        }
        self.insert_reverse(record, counters);
    }

    pub(crate) fn attach_anchor(
        &mut self,
        anchor: AnchorKey,
        record: &SupportRecord,
        counters: &mut WorkCounters,
    ) {
        if let Some(existing) = self.anchor_by_support.get(&record.support_occurrence_ref) {
            debug_assert_eq!(existing, &anchor);
            return;
        }
        self.visible.insert(record.support_occurrence_ref.clone());
        self.by_anchor
            .entry(anchor.clone())
            .or_default()
            .insert(record.support_occurrence_ref.clone());
        self.anchor_by_support
            .insert(record.support_occurrence_ref.clone(), anchor);
        self.insert_reverse(record, counters);
    }

    fn insert_reverse(&mut self, record: &SupportRecord, counters: &mut WorkCounters) {
        self.by_output
            .entry(record.output.clone())
            .or_default()
            .insert(record.support_occurrence_ref.clone());
        for premise in &record.premise_occurrence_refs {
            *self
                .by_premise
                .entry(premise.clone())
                .or_default()
                .entry(record.support_occurrence_ref.clone())
                .or_default() += 1;
        }
        counters.support_entries_written = counters.support_entries_written.saturating_add(1);
    }

    pub(crate) fn detach_anchor(
        &mut self,
        anchor: &AnchorKey,
        records: &BTreeMap<OpaqueRef, SupportRecord>,
        counters: &mut WorkCounters,
    ) {
        let Some(support_refs) = self.by_anchor.remove(anchor) else {
            return;
        };
        for support_ref in support_refs {
            self.anchor_by_support.remove(&support_ref);
            self.visible.remove(&support_ref);
            if let Some(record) = records.get(&support_ref) {
                remove_set_member(&mut self.by_output, &record.output, &support_ref);
                for premise in &record.premise_occurrence_refs {
                    remove_counted_member(&mut self.by_premise, premise, &support_ref);
                }
                counters.support_entries_written =
                    counters.support_entries_written.saturating_add(1);
            }
        }
    }

    pub(crate) fn output_visibility(&self, output: &OpaqueRef) -> bool {
        self.by_output.contains_key(output)
    }

    pub(crate) fn visible_supports_for_anchor(
        &self,
        anchor: &AnchorKey,
    ) -> Option<&BTreeSet<OpaqueRef>> {
        self.by_anchor.get(anchor)
    }

    pub(crate) fn exactly_matches_records(
        &self,
        records: &BTreeMap<OpaqueRef, SupportRecord>,
    ) -> bool {
        self.visible.len() == records.len() && self.visible.iter().eq(records.keys())
    }

    pub(crate) fn sizes(&self) -> ReverseIndexSizes {
        ReverseIndexSizes {
            anchors: self.by_anchor.len(),
            outputs: self.by_output.len(),
            premises: self.by_premise.len(),
            premise_edges: self
                .by_premise
                .values()
                .flat_map(BTreeMap::values)
                .copied()
                .fold(0_u64, u64::saturating_add),
            visible_supports: self.visible.len(),
        }
    }
}

fn remove_set_member<K: Ord + Clone>(
    index: &mut BTreeMap<K, BTreeSet<OpaqueRef>>,
    key: &K,
    value: &OpaqueRef,
) {
    let remove_entry = index.get_mut(key).is_some_and(|values| {
        values.remove(value);
        values.is_empty()
    });
    if remove_entry {
        index.remove(key);
    }
}

fn remove_counted_member(
    index: &mut BTreeMap<OpaqueRef, BTreeMap<OpaqueRef, u64>>,
    key: &OpaqueRef,
    value: &OpaqueRef,
) {
    let remove_entry = index.get_mut(key).is_some_and(|values| {
        values.remove(value);
        values.is_empty()
    });
    if remove_entry {
        index.remove(key);
    }
}

/// Visible reverse-index sizes, including repeated premise multiplicity.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ReverseIndexSizes {
    pub anchors: usize,
    pub outputs: usize,
    pub premises: usize,
    pub premise_edges: u64,
    pub visible_supports: usize,
}
