# Implementation Notes for Missing Routes

## Overview
All route handlers have been created according to the design document specifications. However, many of them depend on db_layer query functions that haven't been implemented yet.

## Missing DB Layer Queries

The following query modules/functions need to be implemented in `db_layer/src/queries/`:

### CollectionQueries (partially implemented)
- `get_books(pool, collection_id, user_id)` - Get books in a collection with metadata

### ReadingStateQueries (not implemented)
- `get_for_book(pool, user_id, book_id)` - Get reading state for a book
- `upsert(pool, user_id, book_id, device_id, location)` - Update reading state
- `get_updated_since(pool, user_id, timestamp)` - Get states updated since timestamp

### AnnotationQueries (not implemented)
- `get_by_id(pool, annotation_id, user_id)` - Get annotation by ID
- `upsert(pool, user_id, book_id, annotation_id, type, location_start, location_end, content, style, deleted)` - Upsert annotation
- `get_updated_since(pool, user_id, timestamp)` - Get annotations updated since timestamp
- `list_for_user(pool, user_id)` - List all user annotations
- `get_for_book(pool, user_id, book_id)` - Get book annotations

### DeviceQueries (needs extension)
- `update_last_sync(pool, device_id, user_id, timestamp)` - Currently only takes device_id and timestamp

### StatsQueries (not implemented)
- `count_users(pool)` - Count total users
- `count_books(pool)` - Count total books
- `count_devices(pool)` - Count total devices
- `count_collections(pool)` - Count total collections
- `count_annotations(pool)` - Count total annotations
- `sum_storage(pool)` - Sum storage used
- `database_size(pool)` - Get database size

## Temporary Solutions

For routes that cannot be fully implemented:
1. Auth and collection routes use proper Create* structs
2. Sync routes are implemented but may fail at runtime until queries exist
3. Admin stats endpoint returns zeros (placeholder)

## Next Steps

To complete the implementation:
1. Implement missing query modules in db_layer
2. Add corresponding database models if needed
3. Update route handlers to handle any API differences
4. Test all endpoints with integration tests
