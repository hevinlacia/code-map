use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::features::notes::NoteRecord;

const MAX_INDEXED_FILES: usize = 80_000;
const MAX_QUERY_FILE_BYTES: u64 = 768 * 1024;
const MAX_SNIPPETS_PER_FILE: usize = 3;
const DEFAULT_MAX_RESULTS: usize = 12;
const MAX_SYMBOL_HITS_PER_FILE: usize = 8;
const MAX_RELATIONSHIP_HITS_PER_FILE: usize = 8;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRecord {
    pub id: Uuid,
    pub name: String,
    pub root_path: String,
    pub indexed: bool,
    pub last_indexed_at: Option<String>,
    pub file_count: u64,
    #[serde(default)]
    pub repo_count: u64,
    pub total_bytes: u64,
    pub symbol_count: u64,
    pub relationship_count: u64,
    #[serde(default)]
    pub constant_fingerprint: Option<String>,
    pub files: Vec<FileRecord>,
    #[serde(default)]
    pub symbols: Vec<SymbolRecord>,
    #[serde(default)]
    pub relationships: Vec<RelationshipRecord>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub id: Uuid,
    pub name: String,
    pub root_path: String,
    pub indexed: bool,
    pub last_indexed_at: Option<String>,
    pub file_count: u64,
    pub repo_count: u64,
    pub total_bytes: u64,
    pub symbol_count: u64,
    pub relationship_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRecord {
    pub relative_path: String,
    #[serde(default)]
    pub repo: Option<String>,
    #[serde(default)]
    pub repo_relative_path: String,
    pub language: Option<String>,
    pub size_bytes: u64,
    #[serde(default)]
    pub mtime: u64,
    pub line_count: Option<u64>,
    pub indexed_content: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolRecord {
    pub kind: String,
    pub name: String,
    pub detail: Option<String>,
    pub relative_path: String,
    pub repo: Option<String>,
    pub repo_relative_path: String,
    pub line: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipRecord {
    pub kind: String,
    pub from: String,
    pub to: String,
    pub detail: Option<String>,
    pub relative_path: String,
    pub repo: Option<String>,
    pub repo_relative_path: String,
    pub line: u64,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub root_path: String,
}

#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub project_id: Option<Uuid>,
    pub query: String,
    pub max_results: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct QueryResponse {
    pub project_id: Uuid,
    pub project_name: String,
    pub query: String,
    pub terms: Vec<String>,
    pub result_count: usize,
    pub summary_lines: Vec<String>,
    pub results: Vec<QueryResult>,
    pub notes: Vec<NoteRecord>,
}

#[derive(Debug, Serialize)]
pub struct QueryResult {
    pub relative_path: String,
    pub repo: Option<String>,
    pub repo_relative_path: String,
    pub language: Option<String>,
    pub score: i64,
    pub reasons: Vec<String>,
    pub snippets: Vec<LineSnippet>,
    pub symbols: Vec<SymbolRecord>,
    pub relationships: Vec<RelationshipRecord>,
}

#[derive(Debug, Serialize)]
pub struct LineSnippet {
    pub line: u64,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct NeighborsRequest {
    pub project_id: Option<Uuid>,
    pub entity: String,
}

#[derive(Debug, Serialize)]
pub struct NeighborsResponse {
    pub project_id: Uuid,
    pub project_name: String,
    pub entity: String,
    pub definitions: Vec<NeighborHit>,
    pub producers: Vec<NeighborHit>,
    pub consumers: Vec<NeighborHit>,
    pub readers: Vec<NeighborHit>,
    pub writers: Vec<NeighborHit>,
    pub callers: Vec<NeighborHit>,
}

#[derive(Debug, Serialize, Clone)]
pub struct NeighborHit {
    pub kind: String,
    pub name: String,
    pub repo: Option<String>,
    pub repo_relative_path: String,
    pub line: u64,
}

#[derive(Debug, Clone)]
struct ScanRoot {
    path: PathBuf,
    repo: Option<String>,
}

struct CollectedFile {
    record: FileRecord,
    contents: Option<String>,
}

struct OldFile {
    record: FileRecord,
    symbols: Vec<SymbolRecord>,
    relationships: Vec<RelationshipRecord>,
}

#[derive(Default)]
struct ScanCollector {
    files: Vec<FileRecord>,
    symbols: Vec<SymbolRecord>,
    relationships: Vec<RelationshipRecord>,
}

struct QueryAccumulator {
    relative_path: String,
    repo: Option<String>,
    repo_relative_path: String,
    language: Option<String>,
    score: i64,
    reasons: Vec<String>,
    snippets: Vec<LineSnippet>,
    symbols: Vec<SymbolRecord>,
    relationships: Vec<RelationshipRecord>,
}

impl ProjectRecord {
    pub fn new(name: impl Into<String>, root_path: impl Into<String>) -> Self {
        let now = now_unix_string();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            root_path: root_path.into(),
            indexed: false,
            last_indexed_at: None,
            file_count: 0,
            repo_count: 0,
            total_bytes: 0,
            symbol_count: 0,
            relationship_count: 0,
            constant_fingerprint: None,
            files: Vec::new(),
            symbols: Vec::new(),
            relationships: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

impl From<&ProjectRecord> for ProjectSummary {
    fn from(project: &ProjectRecord) -> Self {
        Self {
            id: project.id,
            name: project.name.clone(),
            root_path: project.root_path.clone(),
            indexed: project.indexed,
            last_indexed_at: project.last_indexed_at.clone(),
            file_count: project.file_count,
            repo_count: project.repo_count,
            total_bytes: project.total_bytes,
            symbol_count: project.symbol_count,
            relationship_count: project.relationship_count,
        }
    }
}

impl QueryAccumulator {
    fn from_file(file: &FileRecord) -> Self {
        Self {
            relative_path: file.relative_path.clone(),
            repo: file.repo.clone(),
            repo_relative_path: effective_repo_relative_path(file).to_string(),
            language: file.language.clone(),
            score: 0,
            reasons: Vec::new(),
            snippets: Vec::new(),
            symbols: Vec::new(),
            relationships: Vec::new(),
        }
    }

    fn into_result(self) -> QueryResult {
        QueryResult {
            relative_path: self.relative_path,
            repo: self.repo,
            repo_relative_path: self.repo_relative_path,
            language: self.language,
            score: self.score,
            reasons: self.reasons,
            snippets: self.snippets,
            symbols: self.symbols,
            relationships: self.relationships,
        }
    }
}

pub fn canonical_project_path(path: &str) -> AppResult<String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(AppError::bad_request("root_path is required"));
    }

    let path = fs::canonicalize(trimmed)
        .map_err(|error| AppError::bad_request(format!("cannot access root_path: {error}")))?;

    if !path.is_dir() {
        return Err(AppError::bad_request("root_path must be a directory"));
    }

    Ok(path.display().to_string())
}

pub fn scan_project(
    mut project: ProjectRecord,
    data_dir: &Path,
    force: bool,
) -> AppResult<ProjectRecord> {
    let workspace_root = PathBuf::from(&project.root_path);
    if !workspace_root.is_dir() {
        return Err(AppError::bad_request(format!(
            "project root is not a directory: {}",
            project.root_path
        )));
    }

    let scan_roots = discover_scan_roots(&workspace_root)?;
    let repo_count = scan_roots.iter().filter(|root| root.repo.is_some()).count() as u64;
    let old_cache = build_old_cache(&project);
    let incremental = !force && !old_cache.is_empty();
    let reused_constants = if incremental {
        load_constants(data_dir).unwrap_or_default()
    } else {
        HashMap::new()
    };

    let mut collected: Vec<CollectedFile> = Vec::new();
    for scan_root in &scan_roots {
        if collected.len() >= MAX_INDEXED_FILES {
            break;
        }
        scan_dir(
            &workspace_root,
            &scan_root.path,
            &scan_root.path,
            scan_root.repo.as_deref(),
            &old_cache,
            incremental,
            &mut collected,
        )?;
    }

    let constant_map = if incremental {
        reused_constants
    } else {
        let mut map: HashMap<String, String> = HashMap::new();
        for file in &collected {
            if let Some(contents) = &file.contents {
                for (name, value) in find_constants(contents) {
                    map.entry(name).or_insert(value);
                }
            }
        }
        project.constant_fingerprint = Some(constant_fingerprint(&map));
        save_constants(data_dir, &map)?;
        map
    };

    let mut collector = ScanCollector::default();
    for CollectedFile { record, contents } in collected {
        let reuse = incremental
            && old_cache.get(&record.relative_path).is_some_and(|old| {
                old.record.mtime == record.mtime && old.record.size_bytes == record.size_bytes
            });

        if reuse {
            let old = old_cache.get(&record.relative_path).unwrap();
            collector.files.push(old.record.clone());
            collector.symbols.extend(old.symbols.clone());
            collector.relationships.extend(old.relationships.clone());
        } else {
            let (symbols, relationships) = contents
                .as_deref()
                .map(|c| extract_records(&record, c, &constant_map))
                .unwrap_or_default();
            collector.files.push(record);
            collector.symbols.extend(symbols);
            collector.relationships.extend(relationships);
        }
    }

    let total_bytes = collector
        .files
        .iter()
        .map(|file| file.size_bytes)
        .sum::<u64>();

    project.indexed = true;
    project.last_indexed_at = Some(now_unix_string());
    project.file_count = collector.files.len() as u64;
    project.repo_count = repo_count;
    project.total_bytes = total_bytes;
    project.symbol_count = collector.symbols.len() as u64;
    project.relationship_count = collector.relationships.len() as u64;
    project.files = collector.files;
    project.symbols = collector.symbols;
    project.relationships = collector.relationships;
    project.updated_at = now_unix_string();

    Ok(project)
}

pub fn query_project(
    project: &ProjectRecord,
    query: &str,
    max_results: Option<usize>,
    max_summary_lines: u32,
    notes: Vec<NoteRecord>,
) -> AppResult<QueryResponse> {
    let normalized_query = query.trim();
    if normalized_query.is_empty() {
        return Err(AppError::bad_request("query is required"));
    }

    if !project.indexed || project.files.is_empty() {
        return Err(AppError::bad_request(
            "project is not indexed yet; scan it before querying",
        ));
    }

    let terms = extract_terms(normalized_query);
    if terms.is_empty() {
        return Err(AppError::bad_request("query must contain searchable terms"));
    }

    let workspace_root = PathBuf::from(&project.root_path);
    let files_by_path = project
        .files
        .iter()
        .map(|file| (file.relative_path.clone(), file.clone()))
        .collect::<HashMap<_, _>>();
    let mut accumulators = HashMap::<String, QueryAccumulator>::new();

    for file in &project.files {
        let relative_lower = file.relative_path.to_lowercase();
        let repo_relative_lower = effective_repo_relative_path(file).to_lowercase();
        let repo_lower = file.repo.as_deref().unwrap_or_default().to_lowercase();
        let mut file_score = 0_i64;
        let mut file_reasons = Vec::new();
        let mut snippets = Vec::new();

        for term in &terms {
            if should_score_direct_term(term, &terms)
                && !repo_lower.is_empty()
                && repo_lower.contains(term)
            {
                file_score += 14;
                file_reasons.push(format!("repo contains '{term}'"));
            }
            if should_score_direct_term(term, &terms) && repo_relative_lower.contains(term) {
                file_score += 12;
                file_reasons.push(format!("repo path contains '{term}'"));
            } else if should_score_direct_term(term, &terms) && relative_lower.contains(term) {
                file_score += 8;
                file_reasons.push(format!("workspace path contains '{term}'"));
            }
        }

        if file.indexed_content {
            let full_path = workspace_root.join(&file.relative_path);
            if let Ok(contents) = fs::read_to_string(&full_path) {
                for (line_index, line) in contents.lines().enumerate() {
                    let lower_line = line.to_lowercase();
                    let matched_terms = terms
                        .iter()
                        .filter(|term| should_score_content_term(term, &terms))
                        .filter(|term| lower_line.contains(term.as_str()))
                        .count();

                    if matched_terms > 0 {
                        file_score += (matched_terms as i64) * 5;
                        if snippets.len() < MAX_SNIPPETS_PER_FILE {
                            snippets.push(LineSnippet {
                                line: (line_index + 1) as u64,
                                text: compact_line(line),
                            });
                        }
                    }
                }
            }
        }

        if file_score > 0 {
            let entry = entry_for_file(&mut accumulators, file);
            entry.score += file_score;
            entry.reasons.extend(file_reasons);
            if entry.reasons.is_empty() {
                push_unique(&mut entry.reasons, "content match".to_string());
            }
            entry.snippets.extend(snippets);
            if entry.snippets.is_empty() && !entry.reasons.is_empty() {
                push_unique(
                    &mut entry.reasons,
                    "path-only match; read file to inspect details".to_string(),
                );
            }
        }
    }

    for symbol in &project.symbols {
        let (score, reasons) = symbol_match_score(symbol, &terms);
        if score <= 0 {
            continue;
        }
        if let Some(file) = files_by_path.get(&symbol.relative_path) {
            let entry = entry_for_file(&mut accumulators, file);
            entry.score += score;
            entry.reasons.extend(reasons);
            if entry.symbols.len() < MAX_SYMBOL_HITS_PER_FILE {
                entry.symbols.push(symbol.clone());
            }
        }
    }

    for relationship in &project.relationships {
        let (score, reasons) = relationship_match_score(relationship, &terms);
        if score <= 0 {
            continue;
        }
        if let Some(file) = files_by_path.get(&relationship.relative_path) {
            let entry = entry_for_file(&mut accumulators, file);
            entry.score += score;
            entry.reasons.extend(reasons);
            if entry.relationships.len() < MAX_RELATIONSHIP_HITS_PER_FILE {
                entry.relationships.push(relationship.clone());
            }
        }
    }

    let mut results = accumulators
        .into_values()
        .map(QueryAccumulator::into_result)
        .collect::<Vec<_>>();

    for result in &mut results {
        dedupe_strings(&mut result.reasons);
        result.snippets.truncate(MAX_SNIPPETS_PER_FILE);
    }

    results.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| display_result_path(left).cmp(&display_result_path(right)))
    });

    let limit = max_results
        .unwrap_or(DEFAULT_MAX_RESULTS)
        .clamp(1, 50)
        .min(results.len());
    results.truncate(limit);

    let summary_lines = build_summary_lines(project, normalized_query, &results, max_summary_lines);

    Ok(QueryResponse {
        project_id: project.id,
        project_name: project.name.clone(),
        query: normalized_query.to_string(),
        terms,
        result_count: results.len(),
        summary_lines,
        results,
        notes,
    })
}

pub fn neighbors_project(project: &ProjectRecord, entity: &str) -> AppResult<NeighborsResponse> {
    let entity = entity.trim();
    if entity.is_empty() {
        return Err(AppError::bad_request("entity is required"));
    }
    if !project.indexed {
        return Err(AppError::bad_request(
            "project is not indexed yet; scan it before resolving neighbors",
        ));
    }

    let needle = normalize_entity(entity);
    if needle.is_empty() {
        return Err(AppError::bad_request(
            "entity must contain searchable content",
        ));
    }

    let mut definitions = Vec::new();
    for symbol in &project.symbols {
        if entity_matches(&symbol.name, &needle) {
            definitions.push(NeighborHit {
                kind: symbol.kind.clone(),
                name: symbol.name.clone(),
                repo: symbol.repo.clone(),
                repo_relative_path: symbol.repo_relative_path.clone(),
                line: symbol.line,
            });
        }
    }

    let mut producers = Vec::new();
    let mut consumers = Vec::new();
    let mut readers = Vec::new();
    let mut writers = Vec::new();
    let mut callers = Vec::new();

    for relationship in &project.relationships {
        if !entity_matches(&relationship.to, &needle) {
            continue;
        }
        let hit = NeighborHit {
            kind: relationship.kind.clone(),
            name: relationship.from.clone(),
            repo: relationship.repo.clone(),
            repo_relative_path: relationship.repo_relative_path.clone(),
            line: relationship.line,
        };
        match relationship.kind.as_str() {
            "mq_publish" => producers.push(hit),
            "mq_consume" => consumers.push(hit),
            "sql_table_read" => readers.push(hit),
            "sql_table_write" => writers.push(hit),
            "feign_client" | "dubbo_reference" | "frontend_calls_api" => callers.push(hit),
            _ => {}
        }
    }

    for list in [
        &mut producers,
        &mut consumers,
        &mut readers,
        &mut writers,
        &mut callers,
        &mut definitions,
    ] {
        dedupe_neighbor_hits(list);
        list.truncate(25);
    }

    Ok(NeighborsResponse {
        project_id: project.id,
        project_name: project.name.clone(),
        entity: entity.to_string(),
        definitions,
        producers,
        consumers,
        readers,
        writers,
        callers,
    })
}

fn normalize_entity(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .filter(|ch| !matches!(ch, '_' | '-' | '/' | '.' | ':' | ' ' | '\t'))
        .collect()
}

fn entity_matches(value: &str, needle: &str) -> bool {
    let normalized = normalize_entity(value);
    !normalized.is_empty() && (normalized == needle || normalized.contains(needle))
}

fn dedupe_neighbor_hits(hits: &mut Vec<NeighborHit>) {
    let mut seen: Vec<(String, u64, String)> = Vec::new();
    hits.retain(|hit| {
        let key = (hit.repo_relative_path.clone(), hit.line, hit.kind.clone());
        if seen.contains(&key) {
            false
        } else {
            seen.push(key);
            true
        }
    });
}

fn constant_fingerprint(map: &HashMap<String, String>) -> String {
    let mut entries: Vec<(&String, &String)> = map.iter().collect();
    entries.sort();
    entries
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn load_constants(data_dir: &Path) -> Option<HashMap<String, String>> {
    let path = data_dir.join("constants.json");
    let contents = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&contents).ok()
}

fn save_constants(data_dir: &Path, map: &HashMap<String, String>) -> AppResult<()> {
    if let Some(parent) = data_dir.parent() {
        fs::create_dir_all(parent).map_err(|error| AppError::internal(error.to_string()))?;
    }
    let tmp = data_dir.join("constants.json.tmp");
    let body =
        serde_json::to_string_pretty(map).map_err(|error| AppError::internal(error.to_string()))?;
    fs::write(&tmp, body).map_err(|error| AppError::internal(error.to_string()))?;
    fs::rename(&tmp, data_dir.join("constants.json"))
        .map_err(|error| AppError::internal(error.to_string()))?;
    Ok(())
}

fn build_old_cache(project: &ProjectRecord) -> HashMap<String, OldFile> {
    let mut symbols_by_path: HashMap<String, Vec<SymbolRecord>> = HashMap::new();
    for symbol in &project.symbols {
        symbols_by_path
            .entry(symbol.relative_path.clone())
            .or_default()
            .push(symbol.clone());
    }

    let mut relationships_by_path: HashMap<String, Vec<RelationshipRecord>> = HashMap::new();
    for relationship in &project.relationships {
        relationships_by_path
            .entry(relationship.relative_path.clone())
            .or_default()
            .push(relationship.clone());
    }

    let mut cache = HashMap::new();
    for file in &project.files {
        let symbols = symbols_by_path
            .remove(&file.relative_path)
            .unwrap_or_default();
        let relationships = relationships_by_path
            .remove(&file.relative_path)
            .unwrap_or_default();
        cache.insert(
            file.relative_path.clone(),
            OldFile {
                record: file.clone(),
                symbols,
                relationships,
            },
        );
    }
    cache
}

fn discover_scan_roots(workspace_root: &Path) -> AppResult<Vec<ScanRoot>> {
    if is_git_repo(workspace_root) {
        return Ok(vec![ScanRoot {
            path: workspace_root.to_path_buf(),
            repo: workspace_root
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_string),
        }]);
    }

    let mut repos = Vec::new();
    discover_git_repos(workspace_root, workspace_root, &mut repos)?;

    if repos.is_empty() {
        Ok(vec![ScanRoot {
            path: workspace_root.to_path_buf(),
            repo: None,
        }])
    } else {
        repos.sort_by(|left, right| left.repo.cmp(&right.repo));
        Ok(repos)
    }
}

fn discover_git_repos(
    workspace_root: &Path,
    dir: &Path,
    repos: &mut Vec<ScanRoot>,
) -> AppResult<()> {
    let entries = fs::read_dir(dir).map_err(|error| {
        AppError::internal(format!(
            "failed to read directory {}: {error}",
            dir.display()
        ))
    })?;

    for entry in entries {
        let entry = entry.map_err(|error| AppError::internal(error.to_string()))?;
        let file_type = entry
            .file_type()
            .map_err(|error| AppError::internal(error.to_string()))?;
        if !file_type.is_dir() {
            continue;
        }

        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if should_skip_workspace_dir(&name) {
            continue;
        }

        if is_git_repo(&path) {
            repos.push(ScanRoot {
                repo: Some(path_label(workspace_root, &path)?),
                path,
            });
        } else {
            discover_git_repos(workspace_root, &path, repos)?;
        }
    }

    Ok(())
}

fn scan_dir(
    workspace_root: &Path,
    repo_root: &Path,
    dir: &Path,
    repo: Option<&str>,
    old_cache: &HashMap<String, OldFile>,
    incremental: bool,
    collected: &mut Vec<CollectedFile>,
) -> AppResult<()> {
    if collected.len() >= MAX_INDEXED_FILES {
        return Ok(());
    }

    let entries = fs::read_dir(dir).map_err(|error| {
        AppError::internal(format!(
            "failed to read directory {}: {error}",
            dir.display()
        ))
    })?;

    for entry in entries {
        if collected.len() >= MAX_INDEXED_FILES {
            break;
        }

        let entry = entry.map_err(|error| AppError::internal(error.to_string()))?;
        let file_type = entry
            .file_type()
            .map_err(|error| AppError::internal(error.to_string()))?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if file_type.is_dir() {
            if should_skip_repo_dir(&name) {
                continue;
            }
            scan_dir(
                workspace_root,
                repo_root,
                &path,
                repo,
                old_cache,
                incremental,
                collected,
            )?;
        } else if file_type.is_file() {
            if should_skip_file(&name, &path) {
                continue;
            }
            if let Some((record, contents)) = file_record(
                workspace_root,
                repo_root,
                repo,
                &path,
                old_cache,
                incremental,
            )? {
                collected.push(CollectedFile { record, contents });
            }
        }
    }

    Ok(())
}

fn file_record(
    workspace_root: &Path,
    repo_root: &Path,
    repo: Option<&str>,
    path: &Path,
    old_cache: &HashMap<String, OldFile>,
    incremental: bool,
) -> AppResult<Option<(FileRecord, Option<String>)>> {
    let metadata = fs::metadata(path).map_err(|error| AppError::internal(error.to_string()))?;
    let size_bytes = metadata.len();
    let mtime = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let relative_path = path_label(workspace_root, path)?;

    if incremental
        && old_cache
            .get(&relative_path)
            .is_some_and(|old| old.record.mtime == mtime && old.record.size_bytes == size_bytes)
    {
        let old = old_cache.get(&relative_path).unwrap();
        return Ok(Some((old.record.clone(), None)));
    }

    let repo_relative_path = path_label(repo_root, path)?;
    let language = language_for_path(path).map(str::to_string);
    let mut line_count = None;
    let mut indexed_content = false;
    let mut contents = None;

    if size_bytes <= MAX_QUERY_FILE_BYTES
        && is_probably_text_path(path)
        && let Ok(text) = fs::read_to_string(path)
    {
        line_count = Some(text.lines().count() as u64);
        indexed_content = true;
        contents = Some(text);
    }

    Ok(Some((
        FileRecord {
            relative_path,
            repo: repo.map(str::to_string),
            repo_relative_path,
            language,
            size_bytes,
            mtime,
            line_count,
            indexed_content,
        },
        contents,
    )))
}

fn extract_records(
    file: &FileRecord,
    contents: &str,
    constant_map: &HashMap<String, String>,
) -> (Vec<SymbolRecord>, Vec<RelationshipRecord>) {
    let mut symbols = Vec::new();
    let mut relationships = Vec::new();

    for (line_index, line) in contents.lines().enumerate() {
        let line_no = (line_index + 1) as u64;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let lower = trimmed.to_lowercase();

        if let Some((kind, name)) = declared_type(trimmed) {
            push_symbol(&mut symbols, file, kind, name.clone(), trimmed, line_no);
            let lower_name = name.to_lowercase();
            if lower_name.contains("listener")
                || lower_name.contains("consumer")
                || lower_name.contains("producer")
                || lower_name.contains("sender")
                || lower_name.contains("mq")
            {
                push_symbol(
                    &mut symbols,
                    file,
                    "mq_component",
                    name.clone(),
                    trimmed,
                    line_no,
                );
            }
            if lower_name.ends_with("mapper") {
                push_symbol(&mut symbols, file, "mapper", name, trimmed, line_no);
            }
        }

        if is_mapping_annotation(trimmed) {
            let name = quoted_strings(trimmed)
                .into_iter()
                .find(|value| value.starts_with('/'))
                .unwrap_or_else(|| annotation_name(trimmed));
            push_symbol(
                &mut symbols,
                file,
                "controller_route",
                name,
                trimmed,
                line_no,
            );
        }

        if lower.contains("@restcontroller") || lower.contains("@controller") {
            push_symbol(
                &mut symbols,
                file,
                "controller",
                annotation_name(trimmed),
                trimmed,
                line_no,
            );
        }

        if lower.contains("@feignclient") {
            let name = best_quoted(trimmed).unwrap_or_else(|| "FeignClient".to_string());
            push_symbol(
                &mut symbols,
                file,
                "feign_client",
                name.clone(),
                trimmed,
                line_no,
            );
            push_relationship(
                &mut relationships,
                file,
                "feign_client",
                file_key(file),
                name,
                trimmed,
                line_no,
            );
        }

        if lower.contains("@dubboreference") || lower.contains("@reference") {
            let name = best_quoted(trimmed).unwrap_or_else(|| "DubboReference".to_string());
            push_symbol(
                &mut symbols,
                file,
                "dubbo_reference",
                name.clone(),
                trimmed,
                line_no,
            );
            push_relationship(
                &mut relationships,
                file,
                "dubbo_reference",
                file_key(file),
                name,
                trimmed,
                line_no,
            );
        }

        if lower.contains("@dubboservice") {
            let name = best_quoted(trimmed).unwrap_or_else(|| "DubboService".to_string());
            push_symbol(&mut symbols, file, "dubbo_service", name, trimmed, line_no);
        }

        if lower.contains("rocketmqmessagelistener") {
            let topic = attr_quoted(trimmed, &["topic"])
                .or_else(|| resolve_attr_constant(trimmed, "topic", constant_map))
                .or_else(|| best_quoted(trimmed))
                .unwrap_or_else(|| "rocketmq-listener".to_string());
            push_symbol(
                &mut symbols,
                file,
                "mq_consumer",
                topic.clone(),
                trimmed,
                line_no,
            );
            push_relationship(
                &mut relationships,
                file,
                "mq_consume",
                file_key(file),
                topic,
                trimmed,
                line_no,
            );
        }

        if lower.contains("topic") || lower.contains("tag") || lower.contains("group") {
            for quoted in quoted_strings(trimmed) {
                let quoted_lower = quoted.to_lowercase();
                if quoted_lower.contains("topic") {
                    push_symbol(
                        &mut symbols,
                        file,
                        "mq_topic",
                        quoted.clone(),
                        trimmed,
                        line_no,
                    );
                    if is_mq_send_context(&lower) {
                        push_relationship(
                            &mut relationships,
                            file,
                            "mq_publish",
                            file_key(file),
                            quoted,
                            trimmed,
                            line_no,
                        );
                    }
                } else if quoted_lower.contains("tag") {
                    push_symbol(&mut symbols, file, "mq_tag", quoted, trimmed, line_no);
                } else if quoted_lower.contains("group") {
                    push_symbol(&mut symbols, file, "mq_group", quoted, trimmed, line_no);
                }
            }
        }

        if is_mq_context(&lower) {
            for (_constant_name, resolved_value) in resolve_constants_in_line(trimmed, constant_map)
            {
                let value_lower = resolved_value.to_lowercase();
                if value_lower.contains("topic") {
                    push_symbol(
                        &mut symbols,
                        file,
                        "mq_topic",
                        resolved_value.clone(),
                        trimmed,
                        line_no,
                    );
                    if is_mq_send_context(&lower) {
                        push_relationship(
                            &mut relationships,
                            file,
                            "mq_publish",
                            file_key(file),
                            resolved_value,
                            trimmed,
                            line_no,
                        );
                    }
                } else if value_lower.contains("tag") {
                    push_symbol(
                        &mut symbols,
                        file,
                        "mq_tag",
                        resolved_value,
                        trimmed,
                        line_no,
                    );
                } else if value_lower.contains("group") {
                    push_symbol(
                        &mut symbols,
                        file,
                        "mq_group",
                        resolved_value,
                        trimmed,
                        line_no,
                    );
                }
            }
        }

        if (trimmed.starts_with("topic") || trimmed.starts_with("Topic"))
            && let Some(topic) = resolve_attr_constant(trimmed, "topic", constant_map)
                .or_else(|| attr_quoted(trimmed, &["topic"]))
            && topic.to_lowercase().contains("topic")
        {
            push_symbol(
                &mut symbols,
                file,
                "mq_consumer",
                topic.clone(),
                trimmed,
                line_no,
            );
            push_relationship(
                &mut relationships,
                file,
                "mq_consume",
                file_key(file),
                topic,
                trimmed,
                line_no,
            );
        }

        if lower.contains("@mapper") {
            push_symbol(
                &mut symbols,
                file,
                "mapper",
                annotation_name(trimmed),
                trimmed,
                line_no,
            );
        }

        for (kind, table) in sql_tables(trimmed) {
            push_symbol(
                &mut symbols,
                file,
                "db_table",
                table.clone(),
                trimmed,
                line_no,
            );
            push_relationship(
                &mut relationships,
                file,
                kind,
                file_key(file),
                table,
                trimmed,
                line_no,
            );
        }

        if is_frontend_source(file) {
            for quoted in quoted_strings(trimmed) {
                if quoted.starts_with('/') && quoted.len() > 1 {
                    push_symbol(
                        &mut symbols,
                        file,
                        "frontend_api_call",
                        quoted.clone(),
                        trimmed,
                        line_no,
                    );
                    push_relationship(
                        &mut relationships,
                        file,
                        "frontend_calls_api",
                        file_key(file),
                        quoted,
                        trimmed,
                        line_no,
                    );
                }
            }
            if lower.contains("menucode") || lower.contains("buttoncode") {
                for quoted in quoted_strings(trimmed) {
                    push_symbol(
                        &mut symbols,
                        file,
                        "frontend_permission",
                        quoted,
                        trimmed,
                        line_no,
                    );
                }
            }
        }
    }

    (symbols, relationships)
}

fn entry_for_file<'a>(
    accumulators: &'a mut HashMap<String, QueryAccumulator>,
    file: &FileRecord,
) -> &'a mut QueryAccumulator {
    accumulators
        .entry(file.relative_path.clone())
        .or_insert_with(|| QueryAccumulator::from_file(file))
}

fn symbol_match_score(symbol: &SymbolRecord, terms: &[String]) -> (i64, Vec<String>) {
    let mut score = 0;
    let mut reasons = Vec::new();
    for term in terms {
        if should_score_direct_term(term, terms) && symbol.name.to_lowercase().contains(term) {
            score += 28;
            reasons.push(format!("symbol {} contains '{term}'", symbol.kind));
        }
        if should_score_direct_term(term, terms) && symbol.kind.to_lowercase().contains(term) {
            score += 12;
            reasons.push(format!("symbol kind contains '{term}'"));
        }
        if should_score_content_term(term, terms)
            && symbol
                .detail
                .as_deref()
                .unwrap_or_default()
                .to_lowercase()
                .contains(term)
        {
            score += 8;
            reasons.push(format!("symbol detail contains '{term}'"));
        }
        if should_score_direct_term(term, terms)
            && symbol
                .repo
                .as_deref()
                .unwrap_or_default()
                .to_lowercase()
                .contains(term)
        {
            score += 8;
            reasons.push(format!("symbol repo contains '{term}'"));
        }
        if should_score_direct_term(term, terms)
            && symbol.repo_relative_path.to_lowercase().contains(term)
        {
            score += 6;
            reasons.push(format!("symbol path contains '{term}'"));
        }
    }
    (score, reasons)
}

fn relationship_match_score(
    relationship: &RelationshipRecord,
    terms: &[String],
) -> (i64, Vec<String>) {
    let mut score = 0;
    let mut reasons = Vec::new();
    for term in terms {
        if should_score_direct_term(term, terms) && relationship.to.to_lowercase().contains(term) {
            score += 24;
            reasons.push(format!("relationship target contains '{term}'"));
        }
        if should_score_direct_term(term, terms) && relationship.from.to_lowercase().contains(term)
        {
            score += 12;
            reasons.push(format!("relationship source contains '{term}'"));
        }
        if should_score_direct_term(term, terms) && relationship.kind.to_lowercase().contains(term)
        {
            score += 10;
            reasons.push(format!("relationship kind contains '{term}'"));
        }
        if should_score_content_term(term, terms)
            && relationship
                .detail
                .as_deref()
                .unwrap_or_default()
                .to_lowercase()
                .contains(term)
        {
            score += 7;
            reasons.push(format!("relationship detail contains '{term}'"));
        }
        if should_score_direct_term(term, terms)
            && relationship
                .repo_relative_path
                .to_lowercase()
                .contains(term)
        {
            score += 5;
            reasons.push(format!("relationship path contains '{term}'"));
        }
    }
    (score, reasons)
}

fn push_symbol(
    symbols: &mut Vec<SymbolRecord>,
    file: &FileRecord,
    kind: impl Into<String>,
    name: impl Into<String>,
    detail: &str,
    line: u64,
) {
    let name = name.into();
    if name.trim().is_empty() {
        return;
    }
    symbols.push(SymbolRecord {
        kind: kind.into(),
        name,
        detail: Some(compact_line(detail)),
        relative_path: file.relative_path.clone(),
        repo: file.repo.clone(),
        repo_relative_path: effective_repo_relative_path(file).to_string(),
        line,
    });
}

fn push_relationship(
    relationships: &mut Vec<RelationshipRecord>,
    file: &FileRecord,
    kind: impl Into<String>,
    from: impl Into<String>,
    to: impl Into<String>,
    detail: &str,
    line: u64,
) {
    let to = to.into();
    if to.trim().is_empty() {
        return;
    }
    relationships.push(RelationshipRecord {
        kind: kind.into(),
        from: from.into(),
        to,
        detail: Some(compact_line(detail)),
        relative_path: file.relative_path.clone(),
        repo: file.repo.clone(),
        repo_relative_path: effective_repo_relative_path(file).to_string(),
        line,
    });
}

fn declared_type(line: &str) -> Option<(&'static str, String)> {
    for (keyword, kind) in [
        ("class", "class"),
        ("interface", "interface"),
        ("enum", "enum"),
    ] {
        if let Some(name) = word_after_keyword(line, keyword) {
            return Some((kind, name));
        }
    }
    None
}

fn word_after_keyword(line: &str, keyword: &str) -> Option<String> {
    let tokens = line
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'))
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();
    tokens.windows(2).find_map(|window| {
        if window[0] == keyword {
            Some(window[1].to_string())
        } else {
            None
        }
    })
}

fn is_mapping_annotation(line: &str) -> bool {
    [
        "@RequestMapping",
        "@GetMapping",
        "@PostMapping",
        "@PutMapping",
        "@DeleteMapping",
        "@PatchMapping",
    ]
    .iter()
    .any(|annotation| line.contains(annotation))
}

fn annotation_name(line: &str) -> String {
    line.split(|ch: char| ch == '(' || ch.is_whitespace())
        .next()
        .unwrap_or(line)
        .trim_start_matches('@')
        .to_string()
}

fn quoted_strings(line: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\'' && ch != '"' {
            continue;
        }
        let quote = ch;
        let mut value = String::new();
        let mut escaped = false;
        for current in chars.by_ref() {
            if escaped {
                value.push(current);
                escaped = false;
                continue;
            }
            if current == '\\' {
                escaped = true;
                continue;
            }
            if current == quote {
                break;
            }
            value.push(current);
        }
        if !value.is_empty() {
            values.push(value);
        }
    }
    values
}

fn best_quoted(line: &str) -> Option<String> {
    quoted_strings(line)
        .into_iter()
        .find(|value| !value.trim().is_empty())
}

fn is_mq_context(lower: &str) -> bool {
    lower.contains("topic")
        || lower.contains("tag")
        || lower.contains("group")
        || lower.contains("send")
        || lower.contains("consume")
        || lower.contains("listener")
        || lower.contains("rocket")
        || lower.contains("producer")
        || lower.contains("destination")
}

fn is_mq_send_context(lower: &str) -> bool {
    lower.contains("send")
        || lower.contains("producer")
        || lower.contains("convertandsend")
        || lower.contains("rocketmqtemplate")
        || lower.contains("destination")
}

fn is_constant_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() >= 5
        && name
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
        && name.chars().any(|ch| ch.is_ascii_uppercase())
}

fn take_identifier(s: &str) -> String {
    s.chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '$')
        .collect()
}

fn take_qualified_identifier(s: &str) -> String {
    s.chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '$' || *ch == '.')
        .collect()
}

fn find_constants(contents: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in contents.lines() {
        if let Some((name, value)) = find_constant_def(line) {
            out.push((name, value));
        }
    }
    out
}

fn find_constant_def(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    for needle in ["String", "def "] {
        if let Some(pos) = trimmed.find(needle) {
            let after = trimmed[pos + needle.len()..].trim_start();
            let name = take_identifier(after);
            if !is_constant_name(&name) {
                continue;
            }
            let rest = after[name.len()..].trim_start();
            if let Some(value_part) = rest.strip_prefix('=')
                && let Some(value) = best_quoted(value_part.trim_start())
            {
                return Some((name, value));
            }
        }
    }
    None
}

fn resolve_constants_in_line(
    line: &str,
    constant_map: &HashMap<String, String>,
) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for segment in
        line.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '$' || ch == '.'))
    {
        if segment.is_empty() {
            continue;
        }
        let last = segment.rsplit('.').next().unwrap_or(segment);
        if last.is_empty() || !is_constant_name(last) {
            continue;
        }
        if let Some(value) = constant_map.get(last)
            && seen.insert(last.to_string())
        {
            out.push((last.to_string(), value.clone()));
        }
    }
    out
}

fn resolve_attr_constant(
    line: &str,
    attr: &str,
    constant_map: &HashMap<String, String>,
) -> Option<String> {
    let lower = line.to_lowercase();
    let pos = lower.find(attr)?;
    let after = &line[pos + attr.len()..];
    let after_eq = after.trim_start().strip_prefix('=')?;
    let id_part = after_eq.trim_start();
    let ident = take_qualified_identifier(id_part);
    let last = ident.rsplit('.').next().unwrap_or(&ident);
    constant_map.get(last).cloned()
}

fn attr_quoted(line: &str, attrs: &[&str]) -> Option<String> {
    let lower = line.to_lowercase();
    for attr in attrs {
        if let Some(index) = lower.find(&attr.to_lowercase()) {
            let tail = &line[index..];
            if let Some(value) = best_quoted(tail) {
                return Some(value);
            }
        }
    }
    None
}

fn sql_tables(line: &str) -> Vec<(&'static str, String)> {
    let mut tables = Vec::new();
    let lower = line.to_lowercase();
    for (needle, kind) in [
        (" from ", "sql_table_read"),
        (" join ", "sql_table_read"),
        ("insert into ", "sql_table_write"),
        ("update ", "sql_table_write"),
        ("delete from ", "sql_table_write"),
    ] {
        let mut offset = 0;
        while let Some(index) = lower[offset..].find(needle) {
            let start = offset + index + needle.len();
            if let Some(table) = table_token(&line[start..]) {
                tables.push((kind, table));
            }
            offset = start;
        }
    }
    tables
}

fn table_token(value: &str) -> Option<String> {
    let token = value
        .trim_start_matches(|ch: char| ch.is_whitespace() || ch == '`')
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '.' || *ch == '`')
        .collect::<String>()
        .trim_matches('`')
        .to_string();
    if token.len() >= 3 && !matches!(token.as_str(), "select" | "where" | "set") {
        Some(token)
    } else {
        None
    }
}

fn is_frontend_source(file: &FileRecord) -> bool {
    matches!(
        file.language.as_deref(),
        Some("TypeScript" | "TSX" | "JavaScript" | "JSX" | "Vue")
    )
}

fn file_key(file: &FileRecord) -> String {
    match &file.repo {
        Some(repo) if !repo.is_empty() => {
            format!("{}:{}", repo, effective_repo_relative_path(file))
        }
        _ => file.relative_path.clone(),
    }
}

fn path_label(root: &Path, path: &Path) -> AppResult<String> {
    Ok(path
        .strip_prefix(root)
        .map_err(|error| AppError::internal(error.to_string()))?
        .to_string_lossy()
        .replace('\\', "/"))
}

fn is_git_repo(path: &Path) -> bool {
    path.join(".git").exists()
}

fn should_skip_workspace_dir(name: &str) -> bool {
    matches!(
        name,
        ".agents"
            | ".codegraph"
            | ".githooks"
            | ".idea"
            | ".opencode"
            | ".ruff_cache"
            | ".serena"
            | ".vscode"
            | "__pycache__"
            | "logs"
            | "node_modules"
            | "structure"
            | "target"
            | "tmp"
    )
}

fn should_skip_repo_dir(name: &str) -> bool {
    matches!(
        name,
        ".git"
            | ".idea"
            | ".vscode"
            | ".serena"
            | ".codegraph"
            | ".gradle"
            | ".mvn"
            | ".next"
            | ".umi"
            | ".umi-production"
            | ".cache"
            | ".vite"
            | ".worktrees"
            | "node_modules"
            | "target"
            | "dist"
            | "build"
            | "coverage"
            | "__pycache__"
            | ".ruff_cache"
            | "logs"
            | "tmp"
    )
}

fn should_skip_file(name: &str, path: &Path) -> bool {
    if name == ".env" || name.starts_with(".env.") {
        return true;
    }

    if name.starts_with("session-") && path.extension().and_then(|ext| ext.to_str()) == Some("md") {
        return true;
    }

    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("png")
            | Some("jpg")
            | Some("jpeg")
            | Some("gif")
            | Some("webp")
            | Some("ico")
            | Some("pdf")
            | Some("zip")
            | Some("gz")
            | Some("xz")
            | Some("zst")
            | Some("jar")
            | Some("war")
            | Some("class")
            | Some("o")
            | Some("so")
            | Some("dylib")
            | Some("dll")
            | Some("exe")
    )
}

fn is_probably_text_path(path: &Path) -> bool {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => matches!(
            ext.to_ascii_lowercase().as_str(),
            "rs" | "ts"
                | "tsx"
                | "js"
                | "jsx"
                | "java"
                | "groovy"
                | "kt"
                | "py"
                | "go"
                | "rb"
                | "php"
                | "c"
                | "h"
                | "cpp"
                | "hpp"
                | "cs"
                | "sql"
                | "xml"
                | "yaml"
                | "yml"
                | "json"
                | "toml"
                | "md"
                | "txt"
                | "html"
                | "vue"
                | "css"
                | "scss"
                | "less"
                | "sh"
                | "bash"
                | "zsh"
                | "fish"
                | "properties"
                | "gradle"
        ),
        None => true,
    }
}

fn language_for_path(path: &Path) -> Option<&'static str> {
    match path
        .extension()
        .and_then(|ext| ext.to_str())?
        .to_ascii_lowercase()
        .as_str()
    {
        "rs" => Some("Rust"),
        "ts" => Some("TypeScript"),
        "tsx" => Some("TSX"),
        "js" => Some("JavaScript"),
        "jsx" => Some("JSX"),
        "java" => Some("Java"),
        "groovy" => Some("Groovy"),
        "kt" => Some("Kotlin"),
        "py" => Some("Python"),
        "go" => Some("Go"),
        "rb" => Some("Ruby"),
        "php" => Some("PHP"),
        "sql" => Some("SQL"),
        "xml" => Some("XML"),
        "yaml" | "yml" => Some("YAML"),
        "json" => Some("JSON"),
        "toml" => Some("TOML"),
        "md" => Some("Markdown"),
        "html" => Some("HTML"),
        "vue" => Some("Vue"),
        "css" | "scss" | "less" => Some("CSS"),
        "sh" | "bash" | "zsh" | "fish" => Some("Shell"),
        _ => None,
    }
}

fn extract_terms(query: &str) -> Vec<String> {
    let mut terms = Vec::new();
    let normalized = query.trim().to_lowercase();

    push_term(&mut terms, normalized.clone());
    if normalized.contains('_')
        || normalized.contains('-')
        || normalized.contains('/')
        || normalized.contains('.')
    {
        push_term(
            &mut terms,
            normalized
                .replace(['_', '/', '.'], "-")
                .split('-')
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
                .join("-"),
        );
        push_term(
            &mut terms,
            normalized
                .replace(['-', '/', '.'], "_")
                .split('_')
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
                .join("_"),
        );
        push_term(
            &mut terms,
            normalized
                .chars()
                .filter(|ch| !matches!(ch, '_' | '-' | '/' | '.' | ':'))
                .collect::<String>(),
        );
    }

    for term in normalized
        .split(|ch: char| ch.is_whitespace() || matches!(ch, '/' | '\\' | '.' | ':' | '_' | '-'))
        .map(str::trim)
        .filter(|term| term.chars().count() >= 2)
    {
        push_term(&mut terms, term.to_string());
    }

    terms
}

fn push_term(terms: &mut Vec<String>, term: String) {
    let term = term.trim().to_string();
    if term.chars().count() >= 2 && !terms.iter().any(|existing| existing == &term) {
        terms.push(term);
    }
}

fn should_score_direct_term(term: &str, terms: &[String]) -> bool {
    term.chars().count() >= 4
        && !is_generic_query_term(term)
        && !has_more_specific_composite(term, terms)
}

fn should_score_content_term(term: &str, terms: &[String]) -> bool {
    should_score_direct_term(term, terms) || is_composite_term(term)
}

fn has_more_specific_composite(term: &str, terms: &[String]) -> bool {
    terms.iter().any(|candidate| {
        candidate != term
            && candidate.len() > term.len() + 2
            && is_composite_term(candidate)
            && candidate.contains(term)
    })
}

fn is_composite_term(term: &str) -> bool {
    term.contains('_') || term.contains('-') || term.contains('/') || term.contains('.')
}

fn is_generic_query_term(term: &str) -> bool {
    matches!(
        term,
        "wms"
            | "api"
            | "web"
            | "core"
            | "service"
            | "impl"
            | "topic"
            | "tag"
            | "group"
            | "src"
            | "main"
            | "java"
            | "groovy"
            | "com"
            | "yl"
            | "ztocwst"
    )
}

fn build_summary_lines(
    project: &ProjectRecord,
    query: &str,
    results: &[QueryResult],
    max_summary_lines: u32,
) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!(
        "Project '{}' query '{}' returned {} candidate file(s) across {} repo(s).",
        project.name,
        query,
        results.len(),
        project.repo_count
    ));

    let detail_limit = (max_summary_lines as usize).saturating_sub(2).min(8);
    for (index, result) in results.iter().take(detail_limit).enumerate() {
        let line_hint = result
            .snippets
            .first()
            .map(|snippet| format!(":{}", snippet.line))
            .or_else(|| {
                result
                    .symbols
                    .first()
                    .map(|symbol| format!(":{}", symbol.line))
            })
            .or_else(|| {
                result
                    .relationships
                    .first()
                    .map(|rel| format!(":{}", rel.line))
            })
            .unwrap_or_default();
        lines.push(format!(
            "{}. {}{} score={} reason={}",
            index + 1,
            display_result_path(result),
            line_hint,
            result.score,
            result
                .reasons
                .first()
                .cloned()
                .unwrap_or_else(|| "content match".to_string())
        ));
    }

    if results.is_empty() {
        lines.push("No matches found. Try an endpoint path, class name, config key, table name, repo name, or MQ topic.".to_string());
    } else {
        lines
            .push("Suggested agent action: read the top 3 repo/path line hints first.".to_string());
    }

    lines
}

fn display_result_path(result: &QueryResult) -> String {
    match &result.repo {
        Some(repo) if !repo.is_empty() => {
            format!("{}:{}", repo, result.repo_relative_path)
        }
        _ => result.relative_path.clone(),
    }
}

fn effective_repo_relative_path(file: &FileRecord) -> &str {
    if file.repo_relative_path.is_empty() {
        &file.relative_path
    } else {
        &file.repo_relative_path
    }
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

fn dedupe_strings(values: &mut Vec<String>) {
    let mut seen = Vec::<String>::new();
    values.retain(|value| {
        if seen.iter().any(|existing| existing == value) {
            false
        } else {
            seen.push(value.clone());
            true
        }
    });
}

fn compact_line(line: &str) -> String {
    let compact = line.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() > 240 {
        compact.chars().take(237).collect::<String>() + "..."
    } else {
        compact
    }
}

fn now_unix_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}
