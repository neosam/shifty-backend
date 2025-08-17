use std::rc::Rc;

use dioxus::prelude::*;
use tracing::info;
use uuid::Uuid;

use crate::{
    error::ShiftyError,
    loader,
    state::text_template::TextTemplate,
};

use super::{
    config::CONFIG,
    error::ERROR_STORE,
};

#[derive(Clone)]
pub struct TextTemplateStore {
    pub templates: Rc<[TextTemplate]>,
    pub selected_template: Option<TextTemplate>,
    pub filtered_templates: Rc<[TextTemplate]>,
    pub current_filter_type: Option<Rc<str>>,
}

impl Default for TextTemplateStore {
    fn default() -> Self {
        Self {
            templates: Rc::new([]),
            selected_template: None,
            filtered_templates: Rc::new([]),
            current_filter_type: None,
        }
    }
}

pub static TEXT_TEMPLATE_STORE: GlobalSignal<TextTemplateStore> = Signal::global(|| TextTemplateStore::default());

#[derive(Debug)]
pub enum TextTemplateAction {
    LoadTemplates,
    LoadTemplatesByType(String),
    LoadTemplate(Uuid),
    SaveTemplate(TextTemplate),
    UpdateTemplate(Uuid, TextTemplate),
    DeleteTemplate(Uuid),
    ClearSelection,
    ClearFilter,
}

pub async fn load_text_templates() -> Result<(), ShiftyError> {
    info!("Loading text templates");
    let templates = loader::load_text_templates(CONFIG.read().clone()).await?;
    TEXT_TEMPLATE_STORE.write().templates = templates.clone();
    if TEXT_TEMPLATE_STORE.read().current_filter_type.is_none() {
        TEXT_TEMPLATE_STORE.write().filtered_templates = templates;
    }
    info!("Loaded text templates");
    Ok(())
}

pub async fn load_text_templates_by_type(template_type: &str) -> Result<(), ShiftyError> {
    info!("Loading text templates by type: {}", template_type);
    let templates = loader::load_text_templates_by_type(CONFIG.read().clone(), template_type).await?;
    TEXT_TEMPLATE_STORE.write().filtered_templates = templates;
    TEXT_TEMPLATE_STORE.write().current_filter_type = Some(template_type.into());
    info!("Loaded text templates by type");
    Ok(())
}

pub async fn load_text_template(template_id: Uuid) -> Result<(), ShiftyError> {
    info!("Loading text template: {}", template_id);
    let template = loader::load_text_template(CONFIG.read().clone(), template_id).await?;
    TEXT_TEMPLATE_STORE.write().selected_template = Some(template);
    info!("Loaded text template");
    Ok(())
}

pub async fn save_text_template(template: &TextTemplate) -> Result<(), ShiftyError> {
    info!("Saving text template");
    let new_template = loader::save_text_template(CONFIG.read().clone(), template).await?;
    
    let mut store = TEXT_TEMPLATE_STORE.write();
    let mut templates = store.templates.to_vec();
    templates.push(new_template.clone());
    store.templates = templates.into();
    
    if store.current_filter_type.is_none() || 
       store.current_filter_type.as_ref() == Some(&new_template.template_type) {
        let mut filtered = store.filtered_templates.to_vec();
        filtered.push(new_template);
        store.filtered_templates = filtered.into();
    }
    
    info!("Saved text template");
    Ok(())
}

pub async fn update_text_template(template_id: Uuid, template: &TextTemplate) -> Result<(), ShiftyError> {
    info!("Updating text template: {}", template_id);
    let updated_template = loader::update_text_template(CONFIG.read().clone(), template_id, template).await?;
    
    let mut store = TEXT_TEMPLATE_STORE.write();
    
    // Update in main templates list
    let mut templates = store.templates.to_vec();
    if let Some(pos) = templates.iter().position(|t| t.id == template_id) {
        templates[pos] = updated_template.clone();
        store.templates = templates.into();
    }
    
    // Update in filtered templates list
    let mut filtered = store.filtered_templates.to_vec();
    if let Some(pos) = filtered.iter().position(|t| t.id == template_id) {
        filtered[pos] = updated_template.clone();
        store.filtered_templates = filtered.into();
    }
    
    // Update selected template if it's the one being updated
    if let Some(ref selected) = store.selected_template {
        if selected.id == template_id {
            store.selected_template = Some(updated_template);
        }
    }
    
    info!("Updated text template");
    Ok(())
}

pub async fn delete_text_template(template_id: Uuid) -> Result<(), ShiftyError> {
    info!("Deleting text template: {}", template_id);
    loader::delete_text_template(CONFIG.read().clone(), template_id).await?;
    
    let mut store = TEXT_TEMPLATE_STORE.write();
    
    // Remove from main templates list
    let templates: Vec<TextTemplate> = store.templates.iter()
        .filter(|t| t.id != template_id)
        .cloned()
        .collect();
    store.templates = templates.into();
    
    // Remove from filtered templates list
    let filtered: Vec<TextTemplate> = store.filtered_templates.iter()
        .filter(|t| t.id != template_id)
        .cloned()
        .collect();
    store.filtered_templates = filtered.into();
    
    // Clear selection if it's the deleted template
    if let Some(ref selected) = store.selected_template {
        if selected.id == template_id {
            store.selected_template = None;
        }
    }
    
    info!("Deleted text template");
    Ok(())
}

pub fn clear_selection() {
    TEXT_TEMPLATE_STORE.write().selected_template = None;
}

pub fn clear_filter() {
    let store = TEXT_TEMPLATE_STORE.read();
    let templates = store.templates.clone();
    drop(store);
    
    let mut store = TEXT_TEMPLATE_STORE.write();
    store.filtered_templates = templates;
    store.current_filter_type = None;
}

pub async fn generate_custom_report(billing_period_id: Uuid, template_id: Uuid) -> Result<String, ShiftyError> {
    info!("Generating custom report for billing period {} with template {}", billing_period_id, template_id);
    let report = loader::generate_custom_report(CONFIG.read().clone(), billing_period_id, template_id).await?;
    info!("Generated custom report");
    Ok(report)
}

pub async fn handle_text_template_action(action: TextTemplateAction) {
    let result = match action {
        TextTemplateAction::LoadTemplates => load_text_templates().await,
        TextTemplateAction::LoadTemplatesByType(template_type) => {
            load_text_templates_by_type(&template_type).await
        }
        TextTemplateAction::LoadTemplate(template_id) => load_text_template(template_id).await,
        TextTemplateAction::SaveTemplate(template) => save_text_template(&template).await,
        TextTemplateAction::UpdateTemplate(template_id, template) => {
            update_text_template(template_id, &template).await
        }
        TextTemplateAction::DeleteTemplate(template_id) => delete_text_template(template_id).await,
        TextTemplateAction::ClearSelection => {
            clear_selection();
            Ok(())
        }
        TextTemplateAction::ClearFilter => {
            clear_filter();
            Ok(())
        }
    };

    if let Err(error) = result {
        *ERROR_STORE.write() = crate::service::error::ErrorStore {
            error: Some(error.into()),
        };
    }
}