use crate::models::{BaseForm, SynonymGroup, Word, WordInGroup};
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, prelude::*};
use std::path::PathBuf;

pub struct DatabaseBuilder {
    data_dir: PathBuf,
    word_map: HashMap<String, i32>,
    words: Vec<Word>,
    next_word_id: i32,
    base_entries: Vec<BaseForm>,
    synonym_groups: Vec<SynonymGroup>,
    word_in_group: Vec<WordInGroup>,
    next_group_id: i32,
}

impl DatabaseBuilder {
    fn new() -> Self {
        Self {
            data_dir: PathBuf::from("Data"),
            word_map: HashMap::new(),
            words: Vec::new(),
            next_word_id: 1,
            base_entries: Vec::new(),
            synonym_groups: Vec::new(),
            word_in_group: Vec::new(),
            next_group_id: 1,
        }
    }

    fn get_or_set_word_id(&mut self, mut word: String) -> i32 {
        if let Some(&word_id) = self.word_map.get(&word) {
            return word_id;
        }
        let word_id = self.next_word_id;
        word.shrink_to_fit();
        self.words.push(Word { id: word_id, word: word.clone() });
        self.word_map.insert(word, word_id);
        self.next_word_id += 1;
        word_id
    }

    fn create_base_entries(&mut self) -> Result<(), Box<dyn Error>> {
        let file = File::open(self.data_dir.join("odm.txt"))?;
        let reader = BufReader::new(file);
        let mut total_lines = 0usize;

        for line in reader.lines() {
            let line = line?;
            total_lines += 1;

            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            let base_form = parts[0].trim().to_lowercase();
            let base_form_id = self.get_or_set_word_id(base_form);

            let mut unique = HashSet::new();
            for part in parts {
                let word = part.trim().to_lowercase();
                let word_id = self.get_or_set_word_id(word);
                if unique.insert((word_id, base_form_id)) {
                    self.base_entries.push(BaseForm { word_id, base_form_id });
                }
            }
        }

        println!("odm.txt - total lines read: {total_lines}");
        Ok(())
    }

    fn create_synonym_entries(&mut self) -> Result<(), Box<dyn Error>> {
        let file = File::open(self.data_dir.join("th_pl_PL_v2.dat"))?;
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().collect::<Result<Vec<_>, _>>()?;
        println!("th_pl_PL_v2.dat - total lines read: {}", lines.len());

        let mut line_number: usize = 1;

        while line_number < lines.len() {
            let current_line = &lines[line_number];

            if current_line.starts_with('-') {
                line_number += 1;
                continue;
            }

            let header_parts: Vec<&str> = current_line.split('|').collect();
            let head_word = header_parts[0].trim().to_lowercase();
            let head_word_id = self.get_or_set_word_id(head_word);

            let synonym_group_count: usize = if header_parts.len() > 1 {
                header_parts[1].trim().parse().unwrap_or(0)
            } else {
                0
            };

            let group_start = line_number + 1;
            let group_end = (line_number + synonym_group_count).min(lines.len() - 1);

            for group_line_number in group_start..=group_end {
                let current_group_line = &lines[group_line_number];
                let synonyms_in_line: Vec<&str> = current_group_line.split('|').collect();

                let group_meaning = if synonyms_in_line.len() > 1 {
                    synonyms_in_line[1].trim().to_lowercase()
                } else {
                    String::new()
                };

                let mut synonym_ids: Vec<i32> = Vec::new();
                for synonym in &synonyms_in_line {
                    let synonym = synonym.trim();
                    if synonym != "-" && !synonym.is_empty() {
                        let synonym_id = self.get_or_set_word_id(synonym.to_lowercase());
                        synonym_ids.push(synonym_id);
                    }
                }

                synonym_ids.sort();
                synonym_ids.dedup();

                if !synonym_ids.is_empty() {
                    let group_id = self.next_group_id;
                    self.next_group_id += 1;

                    for &sid in &synonym_ids {
                        self.word_in_group.push(WordInGroup { word_id: sid, group_id });
                    }
                    if !synonym_ids.contains(&head_word_id) {
                        self.word_in_group.push(WordInGroup { word_id: head_word_id, group_id });
                    }
                    self.synonym_groups.push(SynonymGroup { group_id, group_meaning });
                }
            }

            line_number = group_end + 1;
        }

        Ok(())
    }

    fn save_to_database(&mut self) -> Result<(), Box<dyn Error>> {
        let mut seen = HashSet::new();
        self.base_entries.retain(|e| seen.insert((e.word_id, e.base_form_id)));

        let database_url = "database.sqlite";
        let mut conn = SqliteConnection::establish(database_url)?;

        conn.batch_execute(
            "PRAGMA journal_mode = MEMORY;
             PRAGMA synchronous = OFF;
             PRAGMA temp_store = MEMORY;"
        )?;

        conn.batch_execute(include_str!("../migrations/2026-05-18-192916-0000_create_tables/up.sql"))?;

        use crate::schema::base_forms::dsl::*;
        use crate::schema::synonym_groups::dsl::*;
        use crate::schema::word_in_group::dsl::*;
        use crate::schema::words::dsl::*;

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            diesel::insert_into(words).values(&self.words).execute(conn)?;

            diesel::insert_into(base_forms).values(&self.base_entries).execute(conn)?;

            let syn_groups: Vec<SynonymGroup> = self.synonym_groups.iter().map(|g| SynonymGroup {
                group_id: g.group_id,
                group_meaning: g.group_meaning.clone(),
            }).collect();

            diesel::insert_into(synonym_groups).values(&syn_groups).execute(conn)?;

            diesel::insert_into(word_in_group).values(&self.word_in_group).execute(conn)?;

            Ok(())
        })?;

        Ok(())
    }

    fn create_database(&mut self) -> Result<(), Box<dyn Error>> {
        self.create_base_entries()?;
        self.create_synonym_entries()?;
        self.save_to_database()?;
        Ok(())
    }
}

pub fn create_database() -> Result<(), Box<dyn Error>> {
    let mut builder = DatabaseBuilder::new();
    builder.create_database()?;
    Ok(())
}
