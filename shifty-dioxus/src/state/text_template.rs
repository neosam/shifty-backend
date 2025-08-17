use std::rc::Rc;
use uuid::Uuid;
use rest_types::{TextTemplateTO, CreateTextTemplateRequestTO, UpdateTextTemplateRequestTO};

#[derive(Clone, Debug, PartialEq)]
pub struct TextTemplate {
    pub id: Uuid,
    pub template_type: Rc<str>,
    pub template_text: Rc<str>,
    pub created_at: Option<time::PrimitiveDateTime>,
    pub created_by: Option<Rc<str>>,
}

impl From<&TextTemplateTO> for TextTemplate {
    fn from(template: &TextTemplateTO) -> Self {
        Self {
            id: template.id,
            template_type: template.template_type.to_string().into(),
            template_text: template.template_text.to_string().into(),
            created_at: template.created_at,
            created_by: template.created_by.as_ref().map(|s| s.to_string().into()),
        }
    }
}

impl TextTemplate {
    pub fn to_create_request(&self) -> CreateTextTemplateRequestTO {
        CreateTextTemplateRequestTO {
            template_type: self.template_type.to_string().into(),
            template_text: self.template_text.to_string().into(),
        }
    }

    pub fn to_update_request(&self) -> UpdateTextTemplateRequestTO {
        UpdateTextTemplateRequestTO {
            template_type: self.template_type.to_string().into(),
            template_text: self.template_text.to_string().into(),
        }
    }
}