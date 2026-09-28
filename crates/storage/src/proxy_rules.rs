use super::{Database, StorageError};
use core_model::proxy_rules::ProxyRule;
use rusqlite::params;

const MIGRATION_005: &str = r#"
CREATE TABLE proxy_rules (
    id TEXT PRIMARY KEY,
    enabled INTEGER NOT NULL,
    priority INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    rule_json TEXT NOT NULL
);
CREATE INDEX idx_proxy_rules_order ON proxy_rules(enabled, priority, created_at, id);
INSERT INTO schema_migrations(version) VALUES (5);
"#;

pub(super) fn initialize(database: &Database) -> Result<(), StorageError> {
    let mut connection = database.connection()?;
    let migrated: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 5)",
        [],
        |row| row.get(0),
    )?;
    if !migrated {
        let transaction = connection.transaction()?;
        transaction.execute_batch(MIGRATION_005)?;
        transaction.commit()?;
    }
    Ok(())
}

impl Database {
    pub fn list_proxy_rules(&self) -> Result<Vec<ProxyRule>, StorageError> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT rule_json FROM proxy_rules ORDER BY priority, created_at, id",
        )?;
        let json = statement.query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        json.iter().map(|value| serde_json::from_str(value).map_err(StorageError::from)).collect()
    }

    pub fn upsert_proxy_rule(&self, rule: &ProxyRule) -> Result<(), StorageError> {
        let connection = self.connection()?;
        let json = serde_json::to_string(rule)?;
        connection.execute(
            "INSERT INTO proxy_rules (id, enabled, priority, created_at, rule_json) VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(id) DO UPDATE SET enabled = excluded.enabled, priority = excluded.priority, created_at = excluded.created_at, rule_json = excluded.rule_json",
            params![rule.id, rule.enabled, rule.priority, rule.created_at, json],
        )?;
        Ok(())
    }

    pub fn delete_proxy_rule(&self, id: &str) -> Result<(), StorageError> {
        self.connection()?.execute("DELETE FROM proxy_rules WHERE id = ?1", [id])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_model::proxy_rules::{ProxyRuleAction, ProxyRuleMatcher, RulePattern, RulePatternKind, PROXY_RULE_SCHEMA_VERSION};

    #[test]
    fn proxy_rules_survive_reopen_in_priority_order() {
        let path = std::env::temp_dir().join(format!("mas-proxy-rules-{}-{}.db", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let database = Database::open(&path).unwrap();
        for (id, priority) in [("later", 2), ("first", 1)] {
            database.upsert_proxy_rule(&ProxyRule {
                schema_version: PROXY_RULE_SCHEMA_VERSION,
                id: id.into(),
                name: id.into(),
                enabled: true,
                priority,
                matcher: ProxyRuleMatcher {
                    method: None,
                    host: RulePattern { kind: RulePatternKind::Wildcard, value: "*".into() },
                    path: RulePattern { kind: RulePatternKind::Wildcard, value: "*".into() },
                },
                action: ProxyRuleAction::Block { status_code: 403 },
                created_at: "1".into(),
                updated_at: "1".into(),
            }).unwrap();
        }
        drop(database);
        let reopened = Database::open(&path).unwrap();
        assert_eq!(reopened.list_proxy_rules().unwrap().iter().map(|rule| rule.id.as_str()).collect::<Vec<_>>(), vec!["first", "later"]);
        std::fs::remove_file(path).unwrap();
    }
}
