# Synonik DB Builder

A Rust CLI tool that downloads Polish thesaurus and inflection data and builds a SQLite database for the [Synonik](https://github.com/leon135/synonik) application.

## Data Sources

| Source | URL | Contents |
|---|---|---|
| **Dobry Słownik** (LibreOffice extension) | `https://pobierz.dobryslownik.pl/pl-dict-latest.oxt` | Polish thesaurus (`th_pl_PL_v2.dat`, `th_pl_PL_v2.idx`), spellchecker, hyphenation patterns |
| **SJP** (Słownik Języka Polskiego) | `https://sjp.pl/sl/odmiany/sjp-odm-*.zip` | Word inflection/declension database (`odm.txt`) |

> **Note:** The SJP download URL includes a date-encoded version. Update the URL in `src/assets_service.rs` if the file becomes unavailable — check `https://sjp.pl/sl/odmiany/` for the latest file.

## Database Schema

```
┌──────────┐    ┌──────────────────┐    ┌──────────────┐
│   words  │    │   base_forms     │    │synonym_groups│
├──────────┤    ├──────────────────┤    ├──────────────┤
│ id (PK)  │◄──►│ word_id (PK,FK)  │    │ group_id (PK)│
│ word     │    │ base_form_id(FK) │───►│ meaning      │
└────┬─────┘    └──────────────────┘    └──────────────┘
     │                                          ▲
     │    ┌──────────────────┐                  │
     └───►│  word_in_group   │──────────────────┘
          ├──────────────────┤
          │ word_id (PK,FK)  │
          │ group_id (PK,FK) │
          └──────────────────┘
```

- **`words`** — every unique word form encountered
- **`base_forms`** — many-to-many link between inflected words and their dictionary base forms
- **`synonym_groups`** — synonym groups with a meaning label
- **`word_in_group`** — many-to-many link between words and synonym groups

## Usage

Run the binary from the project root:

```
cargo run
```

Select an option:

1. **Download database assets** — downloads and extracts the thesaurus `.oxt` file and the SJP inflection `.zip` into `Data/`.
2. **Build database** — parses `Data/odm.txt` and `Data/th_pl_PL_v2.dat`, creates a `database.sqlite` with all four tables populated.

### Environment

The database path can be set via the `DATABASE_URL` environment variable (defaults to `database.sqlite` in the current directory). A `.env` file with `DATABASE_URL=database.sqlite` is also supported.

## Data Directory

After downloading and extracting assets, the `Data/` directory contains:

| File | Source | Used by |
|---|---|---|
| `th_pl_PL_v2.dat` | Dobry Słownik `.oxt` | `create_synonym_entries()` |
| `odm.txt` | SJP `.zip` | `create_base_entries()` |

## How It Works

1. **`download_assets()`** — fetches the `.oxt` extension and `.zip` archive, extracts them into `Data/`.
2. **`create_base_entries()`** — parses `odm.txt` (CSV of word forms where the first token is the base form), creates a mapping from all inflected forms to their base forms.
3. **`create_synonym_entries()`** — parses `th_pl_PL_v2.dat` (OpenOffice thesaurus format), groups synonyms under headwords with meaning labels.
4. **`save_to_database()`** — runs the schema migration and bulk-inserts all words, base form links, synonym groups, and word-group memberships in a single transaction.

## License

Apache License 2.0 — see [LICENSE](LICENSE)  for details.

---

**Built with 💜 by [Leon135](https://leon135.xyz)**
