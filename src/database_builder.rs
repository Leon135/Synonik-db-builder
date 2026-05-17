use crate::models::{BaseFormEntry, SynonymEntry, SynonymGroupEntry, WordEntry};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, prelude::*};

pub struct DatabaseBuilder {
    word_map: HashMap<String, u32>,
    words: Vec<WordEntry>,
    next_word_id: u32,

    base_entries: Vec<BaseFormEntry>,
    next_base_id: u32,

    synonym_entries: Vec<SynonymEntry>,
    next_synonym_id: u32,

    synonym_groups: Vec<SynonymGroupEntry>,
    next_group_id: u32,
}

impl DatabaseBuilder {
    pub fn new() -> Self {
        Self {
            word_map: HashMap::new(),
            words: Vec::new(),
            next_word_id: 1,
            base_entries: Vec::new(),
            next_base_id: 1,
            synonym_entries: Vec::new(),
            next_synonym_id: 1,
            synonym_groups: Vec::new(),
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
                    id: self.next_base_id,
                    word_id,
                    base_form_id,
                });
                self.next_base_id += 1;
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

                let mut synonym_groups: Vec<SynonymGroupEntry> = Vec::new();

                let group_start = line_number + 1;
                let group_end = (line_number + synonym_group_count).min(lines.len() - 1);

                for group_line_number in group_start..=group_end {
                    let mut group_synonyms: SynonymGroupEntry = SynonymGroupEntry { id: self.next_group_id, group_meaning: "test".to_string(), synonyms_ids: Vec::new() };

                    let current_group_line = &lines[group_line_number];
                    let synonyms_in_line: Vec<&str> = current_group_line.split('|').collect();

                    for synonym in synonyms_in_line {
                        if synonym != "-" {
                            let synonym_id = self.get_or_set_word_id(synonym.trim().to_lowercase());
                            group_synonyms.synonyms_ids.push(synonym_id);
                        }
                    }
                    if group_synonyms.synonyms_ids.len() > 0 {
                        synonym_groups.push(group_synonyms);
                        self.next_group_id += 1;
                    }
                    
                }
                
                self.synonym_entries.push(SynonymEntry { id: self.next_synonym_id, word_id: head_word_id, synonyms_groups: synonym_groups });
                self.next_synonym_id += 1;
                line_number = group_end + 1;
            } else {
                line_number += 1;
            }
        }

        Ok(())
    }

    pub fn test_export_json(&mut self) -> Result<(), Box<dyn Error>> {
        self.create_base_entries()?;
        self.create_synonym_entries()?;

        std::fs::write("test_words.json", serde_json::to_string_pretty(&self.words)?)?;
        std::fs::write("test_base_forms.json", serde_json::to_string_pretty(&self.base_entries)?)?;
        std::fs::write("test_synonyms.json", serde_json::to_string_pretty(&self.synonym_entries)?)?;
        std::fs::write("test_synonym_groups.json", serde_json::to_string_pretty(&self.synonym_groups)?)?;

        println!("=== PODSUMOWANIE ===");
        println!("Słowa:     {}", self.words.len());
        println!("Odmiany:   {}", self.base_entries.len());
        println!("Synonimy:  {}", self.synonym_entries.len());
        println!("Grupy:     {}", self.synonym_groups.len());
        println!("\nZapisano do test_*.json");

        Ok(())
    }
}
