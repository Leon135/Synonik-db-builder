#[derive(serde::Serialize)]
pub struct WordEntry {
    pub id: u32,
    pub word: String,
}

#[derive(serde::Serialize)]
pub struct BaseFormEntry {
    pub word_id: u32,
    pub base_form_id: u32,
}

#[derive(serde::Serialize)]
pub struct SynonymGroupEntry {
    pub id: u32,
    pub group_meaning: String,
    pub synonyms_ids: Vec<u32>,
}

#[derive(serde::Serialize)]
pub struct WordInGroupEntry {
    pub word_id: u32,
    pub group_id: u32,
}
