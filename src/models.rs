use crate::schema::*;
use diesel::prelude::*;

#[derive(Debug, Queryable, Insertable)]
#[diesel(table_name = words)]
pub struct Word {
    pub id: i32,
    pub word: String,
}

#[derive(Debug, Queryable, Insertable)]
#[diesel(table_name = base_forms)]
pub struct BaseForm {
    pub word_id: i32,
    pub base_form_id: i32,
}

#[derive(Debug, Queryable, Insertable)]
#[diesel(table_name = synonym_groups)]
pub struct SynonymGroup {
    pub group_id: i32,
    pub group_meaning: String,
}

#[derive(Debug, Queryable, Insertable)]
#[diesel(table_name = word_in_group)]
pub struct WordInGroup {
    pub word_id: i32,
    pub group_id: i32,
}
