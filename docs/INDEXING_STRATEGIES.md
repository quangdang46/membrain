# INDEXING STRATEGIES

This document defines recommended indexes and when to use them.

## 1. Primary id index

### Why this index exists
Primary id index supports a distinct retrieval need and should not be merged blindly with unrelated access patterns.

### Storage notes
- choose write-friendly structures in Tier2
- choose sparse structures in Tier3
- preserve rebuildability from durable logs

### Operational notes
- track hit rate
- track stale index ratio
- track repair backlog
- expose rebuild commands

## 2. Entity inverted index

### Why this index exists
Entity inverted index supports a distinct retrieval need and should not be merged blindly with unrelated access patterns.

### Storage notes
- choose write-friendly structures in Tier2
- choose sparse structures in Tier3
- preserve rebuildability from durable logs

### Operational notes
- track hit rate
- track stale index ratio
- track repair backlog
- expose rebuild commands

## 3. Tag index

### Why this index exists
Tag index supports a distinct retrieval need and should not be merged blindly with unrelated access patterns.

### Storage notes
- choose write-friendly structures in Tier2
- choose sparse structures in Tier3
- preserve rebuildability from durable logs

### Operational notes
- track hit rate
- track stale index ratio
- track repair backlog
- expose rebuild commands

## 4. Session index

### Why this index exists
Session index supports a distinct retrieval need and should not be merged blindly with unrelated access patterns.

### Storage notes
- choose write-friendly structures in Tier2
- choose sparse structures in Tier3
- preserve rebuildability from durable logs

### Operational notes
- track hit rate
- track stale index ratio
- track repair backlog
- expose rebuild commands

## 5. Goal index

### Why this index exists
Goal index supports a distinct retrieval need and should not be merged blindly with unrelated access patterns.

### Storage notes
- choose write-friendly structures in Tier2
- choose sparse structures in Tier3
- preserve rebuildability from durable logs

### Operational notes
- track hit rate
- track stale index ratio
- track repair backlog
- expose rebuild commands

## 6. Time-bucket index

### Why this index exists
Time-bucket index supports a distinct retrieval need and should not be merged blindly with unrelated access patterns.

### Storage notes
- choose write-friendly structures in Tier2
- choose sparse structures in Tier3
- preserve rebuildability from durable logs

### Operational notes
- track hit rate
- track stale index ratio
- track repair backlog
- expose rebuild commands

## 7. Graph adjacency index

### Why this index exists
Graph adjacency index supports a distinct retrieval need and should not be merged blindly with unrelated access patterns.

### Storage notes
- choose write-friendly structures in Tier2
- choose sparse structures in Tier3
- preserve rebuildability from durable logs

### Operational notes
- track hit rate
- track stale index ratio
- track repair backlog
- expose rebuild commands

## 8. ANN index sidecar

### Why this index exists
ANN index sidecar supports a distinct retrieval need and should not be merged blindly with unrelated access patterns.

### Storage notes
- choose write-friendly structures in Tier2
- choose sparse structures in Tier3
- preserve rebuildability from durable logs

### Operational notes
- track hit rate
- track stale index ratio
- track repair backlog
- expose rebuild commands

## 9. Bloom filters

### Why this index exists
Bloom filters supports a distinct retrieval need and should not be merged blindly with unrelated access patterns.

### Storage notes
- choose write-friendly structures in Tier2
- choose sparse structures in Tier3
- preserve rebuildability from durable logs

### Operational notes
- track hit rate
- track stale index ratio
- track repair backlog
- expose rebuild commands

## 10. Prefix indexes

### Why this index exists
Prefix indexes supports a distinct retrieval need and should not be merged blindly with unrelated access patterns.

### Storage notes
- choose write-friendly structures in Tier2
- choose sparse structures in Tier3
- preserve rebuildability from durable logs

### Operational notes
- track hit rate
- track stale index ratio
- track repair backlog
- expose rebuild commands

