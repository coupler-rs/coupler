use crate::{bus::*, editor::*, params::*, process::*};

pub struct PluginInfo {
    pub name: String,
    pub vendor: String,
    pub url: String,
    pub email: String,
    pub has_editor: bool,
}

pub trait Plugin: Send + Sync + Sized {
    type Processor: Processor;
    type Editor: Editor;

    fn info() -> PluginInfo;

    fn create() -> Self;
    fn processor(&self, context: &ProcessContext) -> Self::Processor;
    fn editor(&self, context: EditorContext, parent: Option<&ParentWindow>) -> Self::Editor;

    fn buses() -> BusList;
    fn supports_layout(inputs: &[BusLayout], outputs: &[BusLayout]) -> bool;

    fn params(&self) -> ParamList;
    fn serialize(&self, params: &ParamValues, write: &mut impl std::io::Write) -> Result<(), ()>;
    fn deserialize(
        &self,
        params: &mut ParamValues,
        read: &mut impl std::io::Read,
    ) -> Result<(), ()>;
}
