//! Canonical total render plans bound to exact Model and runtime state Revisions.

use crate::{
    kernel::{
        FiniteF32, KernelError, ReferentId, Result, Revision, RevisionId, RoleId, StructuralForm,
        Term,
    },
    runtime::{StateRevision, StateRevisionId},
    wire::json::{JsonParser, array, json, list, require_string, string},
};

pub const RENDER_PLAN_TAG: &str = "clause-render-plan-v1";

/// The authored relation and roles used by the bounded direct scene
/// projector.  The relation must carry exactly one item referent and one
/// finite two-component tuple position.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SceneProjectionSpec {
    relation: ReferentId,
    item_role: RoleId,
    position_role: RoleId,
    position_shape: ReferentId,
    x_field: ReferentId,
    y_field: ReferentId,
}

impl SceneProjectionSpec {
    pub fn new(
        relation: ReferentId,
        item_role: RoleId,
        position_role: RoleId,
        position_shape: ReferentId,
        x_field: ReferentId,
        y_field: ReferentId,
    ) -> Result<Self> {
        if item_role == position_role {
            return Err(KernelError::new("scene projection roles must be distinct"));
        }
        if x_field == y_field {
            return Err(KernelError::new(
                "scene projection position fields must be distinct",
            ));
        }
        Ok(Self {
            relation,
            item_role,
            position_role,
            position_shape,
            x_field,
            y_field,
        })
    }
}

/// Project only currently supported, grounded scene facts into the canonical
/// total plan. No support root is treated as semantic provenance.
pub fn project_render_plan(
    revision: &Revision,
    state: &StateRevision,
    spec: &SceneProjectionSpec,
) -> Result<RenderPlan> {
    if state.model_revision() != revision.identity() {
        return Err(KernelError::new(
            "scene projection names the wrong Model Revision",
        ));
    }
    let relation = revision
        .model()
        .relation_shapes()
        .get(&spec.relation)
        .ok_or_else(|| KernelError::new("scene projection relation is absent from the Model"))?;
    if relation.roles().len() != 2
        || !relation.roles().contains_key(&spec.item_role)
        || !relation.roles().contains_key(&spec.position_role)
    {
        return Err(KernelError::new(
            "scene projection relation roles do not match",
        ));
    }
    let position_contract = revision
        .model()
        .structural_contracts()
        .get(&spec.position_shape)
        .ok_or_else(|| KernelError::new("scene projection position shape is absent"))?;
    let StructuralForm::Product(fields) = position_contract.form() else {
        return Err(KernelError::new(
            "scene projection position shape is not a labelled product",
        ));
    };
    if fields.len() != 2 || !fields.contains(&spec.x_field) || !fields.contains(&spec.y_field) {
        return Err(KernelError::new(
            "scene projection position shape does not match x/y fields",
        ));
    }
    let mut items = Vec::new();
    for content in state.supported_contents() {
        if content.relation() != &spec.relation {
            continue;
        }
        if !content.is_ground() {
            return Err(KernelError::new(
                "scene projection requires grounded content",
            ));
        }
        if content.roles().len() != 2
            || !content.roles().contains_key(&spec.item_role)
            || !content.roles().contains_key(&spec.position_role)
        {
            return Err(KernelError::new(
                "scene projection relation roles do not match",
            ));
        }
        let item = match content.roles().get(&spec.item_role) {
            Some(Term::Referent(id)) => id.clone(),
            _ => return Err(KernelError::new("scene projection item is not a referent")),
        };
        let position = position_f32x2(content.roles().get(&spec.position_role), spec)?;
        items.push(RenderItem::new(item, position)?);
    }
    items.sort_by(|left, right| left.id().cmp(right.id()));
    RenderPlan::new(revision, state, items)
}

fn position_f32x2(term: Option<&Term>, spec: &SceneProjectionSpec) -> Result<[FiniteF32; 2]> {
    let Some(Term::LabelledProduct { shape, fields }) = term else {
        return Err(KernelError::new(
            "scene projection position is not the labelled Vec2 shape",
        ));
    };
    if shape != &spec.position_shape
        || fields.len() != 2
        || !fields.contains_key(&spec.x_field)
        || !fields.contains_key(&spec.y_field)
    {
        return Err(KernelError::new(
            "scene projection position does not match its exact Vec2 shape",
        ));
    }
    let component = |field: &ReferentId| match fields.get(field) {
        Some(Term::F32(value)) => Ok(*value),
        _ => Err(KernelError::new(
            "scene projection position component is not F32",
        )),
    };
    Ok([component(&spec.x_field)?, component(&spec.y_field)?])
}

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
