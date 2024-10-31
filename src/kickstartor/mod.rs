use crate::GaladrielResult;

#[derive(Clone, PartialEq, Debug)]
pub struct Kickstartor {
    central_context: Vec<String>,
    layout_contexts: Vec<String>,
    module_contexts: Vec<String>,

    is_names_on_save: bool,
    exclude: Vec<String>,
}

impl Kickstartor {
    pub fn new(exclude: Vec<String>, is_names_on_save: bool) -> Self {
        Self {
            exclude,
            is_names_on_save,
            central_context: vec![],
            layout_contexts: vec![],
            module_contexts: vec![],
        }
    }

    pub async fn process_nyr_files(&mut self) -> GaladrielResult<()> {
        Ok(())
    }
}
