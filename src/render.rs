//! Canonical total render plans bound to exact Model and runtime state Revisions.

use crate::{
    kernel::{FiniteF32, KernelError, ReferentId, Result, Revision, RevisionId},
    runtime::{StateRevision, StateRevisionId},
    wire::json::{JsonParser, array, json, list, require_string, string},
};

pub const RENDER_PLAN_TAG: &str = "clause-render-plan-v1";

/// One desired scene item in an exact state-bound snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderItem {
    id: ReferentId,
    position: [FiniteF32; 2],
}

impl RenderItem {
    pub fn new(id: ReferentId, position: [FiniteF32; 2]) -> Result<Self> {
        Ok(Self { id, position })
    }

    pub fn id(&self) -> &ReferentId {
        &self.id
    }

    pub fn position(&self) -> [FiniteF32; 2] {
        self.position
    }

    fn canonical_bytes(&self) -> String {
        format!(
            "[\"item\",\"{}\",[\"position-f32x2\",\"{:08x}\",\"{:08x}\"]]",
            self.id.as_str(),
            self.position[0].bits(),
            self.position[1].bits(),
        )
    }
}

/// The complete desired scene for one exact immutable runtime state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderPlan {
    model_revision: RevisionId,
    state_revision: StateRevisionId,
    items: Vec<RenderItem>,
}

impl RenderPlan {
    pub fn new(revision: &Revision, state: &StateRevision, items: Vec<RenderItem>) -> Result<Self> {
        if state.model_revision() != revision.identity() {
            return Err(KernelError::new(
                "RenderPlan state names the wrong Model Revision",
            ));
        }
        if items.windows(2).any(|pair| pair[0].id >= pair[1].id) {
            return Err(KernelError::new(
                "RenderPlan items must be strictly canonical",
            ));
        }
        for item in &items {
            if !revision.model().referents().contains_key(item.id()) {
                return Err(KernelError::new(
                    "RenderPlan item identity is absent from the checked Model",
                ));
            }
        }
        Ok(Self {
            model_revision: revision.identity().clone(),
            state_revision: state.identity().clone(),
            items,
        })
    }

    pub fn model_revision(&self) -> &RevisionId {
        &self.model_revision
    }

    pub fn state_revision(&self) -> &StateRevisionId {
        &self.state_revision
    }

    pub fn items(&self) -> &[RenderItem] {
        &self.items
    }

    pub fn canonical_bytes(&self) -> String {
        let items = self
            .items
            .iter()
            .map(RenderItem::canonical_bytes)
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "[\"{RENDER_PLAN_TAG}\",[\"model-revision\",\"{}\"],[\"state-revision\",\"{}\"],[\"items\",[{items}]]]",
            self.model_revision, self.state_revision,
        )
    }
}

/// Strictly reload a canonical plan against its exact Model and state Revisions.
pub fn reload_render_plan(
    bytes: &str,
    revision: &Revision,
    state: &StateRevision,
) -> Result<RenderPlan> {
    let value = JsonParser::new(bytes).parse()?;
    if json(&value) != bytes {
        return Err(KernelError::new("RenderPlan wire is not canonical JSON"));
    }
    let envelope = list(&value, 4, "RenderPlan envelope")?;
    require_string(&envelope[0], RENDER_PLAN_TAG, "RenderPlan tag")?;

    let model = list(&envelope[1], 2, "RenderPlan Model Revision")?;
    require_string(&model[0], "model-revision", "RenderPlan Model Revision tag")?;
    if string(&model[1], "RenderPlan Model Revision identity")? != revision.identity().to_string() {
        return Err(KernelError::new(
            "RenderPlan names the wrong Model Revision",
        ));
    }

    let exact_state = list(&envelope[2], 2, "RenderPlan StateRevision")?;
    require_string(
        &exact_state[0],
        "state-revision",
        "RenderPlan StateRevision tag",
    )?;
    if string(&exact_state[1], "RenderPlan StateRevision identity")? != state.identity().as_str() {
        return Err(KernelError::new("RenderPlan names the wrong StateRevision"));
    }

    let item_field = list(&envelope[3], 2, "RenderPlan items field")?;
    require_string(&item_field[0], "items", "RenderPlan items tag")?;
    let items = array(&item_field[1], "RenderPlan items")?
        .iter()
        .map(decode_item)
        .collect::<Result<Vec<_>>>()?;
    let plan = RenderPlan::new(revision, state, items)?;
    if plan.canonical_bytes() != bytes {
        return Err(KernelError::new(
            "RenderPlan does not match its exact canonical content",
        ));
    }
    Ok(plan)
}

fn decode_item(value: &crate::wire::json::Json) -> Result<RenderItem> {
    let item = list(value, 3, "RenderPlan item")?;
    require_string(&item[0], "item", "RenderPlan item tag")?;
    let id = ReferentId::new(string(&item[1], "RenderPlan item identity")?.to_owned())?;

    let position = list(&item[2], 3, "RenderPlan position")?;
    require_string(&position[0], "position-f32x2", "RenderPlan position tag")?;
    let x = decode_f32(&position[1])?;
    let y = decode_f32(&position[2])?;
    RenderItem::new(id, [x, y])
}

fn decode_f32(value: &crate::wire::json::Json) -> Result<FiniteF32> {
    let bits = string(value, "RenderPlan F32 bits")?;
    if bits.len() != 8 {
        return Err(KernelError::new("invalid RenderPlan F32 bits"));
    }
    let bits = u32::from_str_radix(bits, 16)
        .map_err(|_| KernelError::new("invalid RenderPlan F32 bits"))?;
    FiniteF32::from_bits(bits)
}
