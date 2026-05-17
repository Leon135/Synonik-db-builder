#[derive(serde::Serialize)]
pub struct WordEntry {
    pub id: u32,
    pub word: String
}

#[derive(serde::Serialize)]
pub struct BaseFormEntry {
    pub id: u32,
    pub word_id: u32,
    pub base_form_id: u32
}

#[derive(serde::Serialize)]
pub struct SynonymEntry {
    pub id: u32,
    pub word_id: u32,
    pub synonyms_groups: Vec<SynonymGroupEntry>
}

#[derive(serde::Serialize)]
pub struct SynonymGroupEntry {
    pub id: u32,
    pub group_meaning: String,
    pub synonyms_ids: Vec<u32>
}