---
name: sqlite-db
description: Methods and SQL commands to inspect the local database state in pvm.db.
---

# SQLite DB Inspector Skill

This skill provides directions to inspect, query, and verify the data stored inside the local SQLite database file `pvm.db` (usually located in the executable folder).

## Common Queries

If you have SQLite CLI installed, you can inspect the schema and rows.

### Show tables
```sql
.tables
```
Expected output:
* `PhpVersions`
* `InstallUrls`

### Show schema of tables
```sql
.schema PhpVersions
.schema InstallUrls
```

### Inspect registered PHP versions
```sql
SELECT Id, Version, Path, IsCurrent FROM PhpVersions;
```

### Inspect current active PHP version
```sql
SELECT * FROM PhpVersions WHERE IsCurrent = 1;
```

### Check downloaded URL cache
```sql
SELECT * FROM InstallUrls LIMIT 10;
```
