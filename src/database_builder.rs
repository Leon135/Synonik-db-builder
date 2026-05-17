use crate::models::{BaseFormEntry, SynonymGroupEntry, WordEntry, WordInGroupEntry};
use rusqlite::Result;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, prelude::*};

pub struct DatabaseBuilder {
    word_map: HashMap<String, u32>,
    words: Vec<WordEntry>,
    next_word_id: u32,

    base_entries: Vec<BaseFormEntry>,

    synonym_groups: Vec<SynonymGroupEntry>,
    word_in_group: Vec<WordInGroupEntry>,
    next_group_id: u32,
}

impl DatabaseBuilder {
    pub fn new() -> Self {
        Self {
            word_map: HashMap::new(),
            words: Vec::new(),
            next_word_id: 1,
            base_entries: Vec::new(),
            synonym_groups: Vec::new(),
            word_in_group: Vec::new(),
            next_group_id: 1,
        }
    }

    fn read_lines(&self, file_path: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().collect::<Result<Vec<_>, _>>()?;

        println!(
            "{file} - total lines read: {lines_count}",
            file = file_path,
            lines_count = lines.len()
        );

        return Ok(lines);
    }

    fn get_or_set_word_id(&mut self, mut word: String) -> u32 {
        let word_id: u32;

        if self.word_map.contains_key(&word) {
            word_id = self.word_map[&word];
        } else {
            word_id = self.next_word_id;
            word.shrink_to_fit();
            self.words.push(WordEntry {
                id: word_id,
                word: word.clone(),
            });
            self.word_map.insert(word, word_id);
            self.next_word_id += 1
        }

        return word_id;
    }

    fn create_base_entries(&mut self) -> Result<(), Box<dyn Error>> {
        let lines = self.read_lines("Data/odm.txt")?;

        for line in lines {
            let parts: Vec<&str> = line.split(',').collect();

            if parts.is_empty() {
                continue;
            }

            let base_form = parts[0].trim().to_lowercase();
            let base_form_id = self.get_or_set_word_id(base_form);

            for part in parts {
                let word = part.trim().to_lowercase();

                let word_id = self.get_or_set_word_id(word);

                self.base_entries.push(BaseFormEntry {
                    word_id,
                    base_form_id,
                });
            }
        }

        Ok(())
    }

    fn create_synonym_entries(&mut self) -> Result<(), Box<dyn Error>> {
        let lines = self.read_lines("Data/th_pl_PL_v2.dat")?;

        let mut line_number: usize = 1;

        while line_number < lines.len() {
            let current_line = &lines[line_number];

            if !current_line.starts_with('-') {
                let header_parts: Vec<&str> = current_line.split('|').collect();
                let head_word = header_parts[0].trim().to_lowercase();
                let head_word_id = self.get_or_set_word_id(head_word);

                let mut synonym_group_count: usize = 0;
                if header_parts.len() > 1 {
                    synonym_group_count = header_parts[1].trim().parse::<usize>().unwrap_or(0);
                }

                let group_start = line_number + 1;
                let group_end = (line_number + synonym_group_count).min(lines.len() - 1);

                for group_line_number in group_start..=group_end {
                    let current_group_line = &lines[group_line_number];
                    let synonyms_in_line: Vec<&str> = current_group_line.split('|').collect();

                    let group_meaning = synonyms_in_line[1].trim().to_lowercase();

                    let mut synonym_ids: Vec<u32> = Vec::new();
                    for synonym in synonyms_in_line {
                        if synonym != "-" {
                            let synonym_id = self.get_or_set_word_id(synonym.trim().to_lowercase());
                            synonym_ids.push(synonym_id);
                        }
                    }

                    synonym_ids.sort();
                    synonym_ids.dedup();

                    if !synonym_ids.is_empty() {
                        let group_id = self.next_group_id;
                        self.next_group_id += 1;

                        for &sid in &synonym_ids {
                            self.word_in_group.push(WordInGroupEntry {
                                word_id: sid,
                                group_id,
                            });
                        }
                        if !synonym_ids.contains(&head_word_id) {
                            self.word_in_group.push(WordInGroupEntry {
                                word_id: head_word_id,
                                group_id,
                            });
                        }
                        self.synonym_groups.push(SynonymGroupEntry {
                            id: group_id,
                            group_meaning: group_meaning,
                            synonyms_ids: synonym_ids,
                        });
                    }
                }

                line_number = group_end + 1;
            } else {
                line_number += 1;
            }
        }

        Ok(())
    }

    fn save_to_database(&self) -> Result<(), Box<dyn Error>> {
        let mut connection = rusqlite::Connection::open("synonik.db")?;

        connection.execute_batch(
            "PRAGMA journal_mode = MEMORY;
             PRAGMA synchronous = OFF;
             PRAGMA temp_store = MEMORY;",
        )?;

        connection.execute(
            "CREATE TABLE IF NOT EXISTS Words (id INTEGER PRIMARY KEY, word TEXT NOT NULL);",
            (),
        )?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS BaseForms (word_id INTEGER PRIMARY KEY, base_form_id INTEGER NOT NULL);",
            (),
        )?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS SynonymGroups (id INTEGER PRIMARY KEY, group_meaning TEXT NOT NULL);",
            (),
        )?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS WordInGroup (word_id INTEGER NOT NULL, group_id INTEGER NOT NULL, PRIMARY KEY (word_id, group_id));",
            (),
        )?;

        let tx = connection.transaction()?;

        {
            let mut stmt = tx.prepare("INSERT INTO Words (id, word) VALUES (?1, ?2)")?;
            for word in &self.words {
                stmt.execute((word.id, &word.word))?;
            }
        }

        {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO BaseForms (word_id, base_form_id) VALUES (?1, ?2)",
            )?;
            for entry in &self.base_entries {
                stmt.execute((entry.word_id, entry.base_form_id))?;
            }
        }

        {
            let mut stmt =
                tx.prepare("INSERT INTO SynonymGroups (id, group_meaning) VALUES (?1, ?2)")?;
            for g in &self.synonym_groups {
                stmt.execute((g.id, &g.group_meaning))?;
            }
        }

        {
            let mut stmt =
                tx.prepare("INSERT INTO WordInGroup (word_id, group_id) VALUES (?1, ?2)")?;
            for wig in &self.word_in_group {
                stmt.execute((wig.word_id, wig.group_id))?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    pub fn create_database(&mut self) -> Result<(), Box<dyn Error>> {
        self.create_base_entries()?;
        self.create_synonym_entries()?;
        self.save_to_database()?;

        Ok(())
    }
}
