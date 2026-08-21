use crate::ir::Value;
use super::PrimitiveError;

pub fn list_new(_args: &[Value]) -> Result<Value, PrimitiveError> {
    Ok(Value::List(vec![]))
}

pub fn list_push(args: &[Value]) -> Result<Value, PrimitiveError> {
    let list = args.first().cloned().ok_or_else(|| PrimitiveError("list_push requires list".into()))?;
    let item = args.get(1).cloned().ok_or_else(|| PrimitiveError("list_push requires item".into()))?;
    match list {
        Value::List(mut items) => { items.push(item); Ok(Value::List(items)) }
        _ => Err(PrimitiveError("list_push: first arg must be list".into())),
    }
}

pub fn list_length(args: &[Value]) -> Result<Value, PrimitiveError> {
    let list = args.first().and_then(|v| match v { Value::List(l) => Some(l), _ => None })
        .ok_or_else(|| PrimitiveError("list_length requires list".into()))?;
    Ok(Value::Int(list.len() as i64))
}

pub fn list_contains(args: &[Value]) -> Result<Value, PrimitiveError> {
    let list = args.first().and_then(|v| match v { Value::List(l) => Some(l), _ => None })
        .ok_or_else(|| PrimitiveError("list_contains requires list".into()))?;
    let item = args.get(1).cloned().ok_or_else(|| PrimitiveError("list_contains requires item".into()))?;
    Ok(Value::Bool(list.contains(&item)))
}

pub fn list_sort(args: &[Value]) -> Result<Value, PrimitiveError> {
    let list = args.first().and_then(|v| match v { Value::List(l) => Some(l.clone()), _ => None })
        .ok_or_else(|| PrimitiveError("list_sort requires list".into()))?;
    let mut sorted = list;
    sorted.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
    Ok(Value::List(sorted))
}

pub fn list_reverse(args: &[Value]) -> Result<Value, PrimitiveError> {
    let list = args.first().and_then(|v| match v { Value::List(l) => Some(l.clone()), _ => None })
        .ok_or_else(|| PrimitiveError("list_reverse requires list".into()))?;
    let mut reversed = list;
    reversed.reverse();
    Ok(Value::List(reversed))
}

pub fn list_unique(args: &[Value]) -> Result<Value, PrimitiveError> {
    let list = args.first().and_then(|v| match v { Value::List(l) => Some(l), _ => None })
        .ok_or_else(|| PrimitiveError("list_unique requires list".into()))?;
    let mut seen = std::collections::HashSet::new();
    let unique: Vec<Value> = list.iter().filter(|item| seen.insert(format!("{item:?}"))).cloned().collect();
    Ok(Value::List(unique))
}

pub fn list_flatten(args: &[Value]) -> Result<Value, PrimitiveError> {
    let list = args.first().and_then(|v| match v { Value::List(l) => Some(l), _ => None })
        .ok_or_else(|| PrimitiveError("list_flatten requires list".into()))?;
    let mut flat = Vec::new();
    for item in list {
        match item {
            Value::List(inner) => flat.extend(inner.iter().cloned()),
            other => flat.push(other.clone()),
        }
    }
    Ok(Value::List(flat))
}

pub fn list_slice(args: &[Value]) -> Result<Value, PrimitiveError> {
    let list = args.first().and_then(|v| match v { Value::List(l) => Some(l), _ => None })
        .ok_or_else(|| PrimitiveError("list_slice requires list".into()))?;
    let start = args.get(1).and_then(|v| match v { Value::Int(n) => Some(*n as usize), _ => None }).unwrap_or(0);
    let end = args.get(2).and_then(|v| match v { Value::Int(n) => Some(*n as usize), _ => None }).unwrap_or(list.len());
    let slice: Vec<Value> = list.iter().skip(start).take(end.saturating_sub(start)).cloned().collect();
    Ok(Value::List(slice))
}

pub fn list_filter(args: &[Value]) -> Result<Value, PrimitiveError> {
    let list = args.first().and_then(|v| match v { Value::List(l) => Some(l), _ => None })
        .ok_or_else(|| PrimitiveError("list_filter requires list".into()))?;
    let keep_bools = args.get(1).and_then(|v| match v { Value::List(l) => Some(l), _ => None });
    if let Some(bools) = keep_bools {
        let filtered: Vec<Value> = list.iter().zip(bools.iter())
            .filter(|(_, b)| matches!(b, Value::Bool(true)))
            .map(|(item, _)| item.clone())
            .collect();
        return Ok(Value::List(filtered));
    }
    Ok(Value::List(list.clone()))
}

pub fn list_map_op(args: &[Value]) -> Result<Value, PrimitiveError> {
    let list = args.first().and_then(|v| match v { Value::List(l) => Some(l), _ => None })
        .ok_or_else(|| PrimitiveError("list_map requires list".into()))?;
    let mapped = args.get(1).and_then(|v| match v { Value::List(l) => Some(l), _ => None });
    if let Some(m) = mapped {
        return Ok(Value::List(m.clone()));
    }
    Ok(Value::List(list.clone()))
}

pub fn list_head(args: &[Value]) -> Result<Value, PrimitiveError> {
    let list = args.first().and_then(|v| match v { Value::List(l) => Some(l), _ => None })
        .ok_or_else(|| PrimitiveError("list_head requires list".into()))?;
    list.first().cloned().ok_or_else(|| PrimitiveError("list_head: empty list".into()))
}

pub fn list_tail(args: &[Value]) -> Result<Value, PrimitiveError> {
    let list = args.first().and_then(|v| match v { Value::List(l) => Some(l), _ => None })
        .ok_or_else(|| PrimitiveError("list_tail requires list".into()))?;
    if list.is_empty() { return Ok(Value::List(vec![])); }
    Ok(Value::List(list[1..].to_vec()))
}

pub fn list_pop(args: &[Value]) -> Result<Value, PrimitiveError> {
    let list = args.first().and_then(|v| match v { Value::List(l) => Some(l.clone()), _ => None })
        .ok_or_else(|| PrimitiveError("list_pop requires list".into()))?;
    list.last().cloned().ok_or_else(|| PrimitiveError("list_pop: empty list".into()))
}

pub fn list_index(args: &[Value]) -> Result<Value, PrimitiveError> {
    let list = args.first().and_then(|v| match v { Value::List(l) => Some(l), _ => None })
        .ok_or_else(|| PrimitiveError("list_index requires list".into()))?;
    let item = args.get(1).cloned().ok_or_else(|| PrimitiveError("list_index requires item".into()))?;
    match list.iter().position(|x| x == &item) {
        Some(i) => Ok(Value::Int(i as i64)),
        None => Ok(Value::Int(-1)),
    }
}

pub fn list_diff(args: &[Value]) -> Result<Value, PrimitiveError> {
    let a = args.first().and_then(|v| match v { Value::List(l) => Some(l), _ => None })
        .ok_or_else(|| PrimitiveError("list_diff requires list a".into()))?;
    let b = args.get(1).and_then(|v| match v { Value::List(l) => Some(l), _ => None })
        .ok_or_else(|| PrimitiveError("list_diff requires list b".into()))?;
    let diff: Vec<Value> = a.iter().filter(|item| !b.contains(item)).cloned().collect();
    Ok(Value::List(diff))
}

pub fn list_sum(args: &[Value]) -> Result<Value, PrimitiveError> {
    let list = args.first().and_then(|v| match v { Value::List(l) => Some(l), _ => None })
        .ok_or_else(|| PrimitiveError("list_sum requires list".into()))?;
    let sum: i64 = list.iter().filter_map(|v| match v { Value::Int(n) => Some(*n), _ => None }).sum();
    Ok(Value::Int(sum))
}

// ============================================================================
// Map operations
// ============================================================================

pub fn map_new(_args: &[Value]) -> Result<Value, PrimitiveError> {
    Ok(Value::Map(vec![]))
}

pub fn map_get(args: &[Value]) -> Result<Value, PrimitiveError> {
    let map = args.first().and_then(|v| match v { Value::Map(m) => Some(m), _ => None })
        .ok_or_else(|| PrimitiveError("map_get requires map".into()))?;
    let key = args.get(1).cloned().ok_or_else(|| PrimitiveError("map_get requires key".into()))?;
    map.iter().find(|(k, _)| k == &key).map(|(_, v)| v.clone())
        .ok_or_else(|| PrimitiveError(format!("map_get: key not found")))
}

pub fn map_set(args: &[Value]) -> Result<Value, PrimitiveError> {
    let map = args.first().and_then(|v| match v { Value::Map(m) => Some(m.clone()), _ => None })
        .ok_or_else(|| PrimitiveError("map_set requires map".into()))?;
    let key = args.get(1).cloned().ok_or_else(|| PrimitiveError("map_set requires key".into()))?;
    let value = args.get(2).cloned().ok_or_else(|| PrimitiveError("map_set requires value".into()))?;
    let mut entries = map;
    entries.retain(|(k, _)| k != &key);
    entries.push((key, value));
    Ok(Value::Map(entries))
}

pub fn map_keys(args: &[Value]) -> Result<Value, PrimitiveError> {
    let map = args.first().and_then(|v| match v { Value::Map(m) => Some(m), _ => None })
        .ok_or_else(|| PrimitiveError("map_keys requires map".into()))?;
    let keys: Vec<Value> = map.iter().map(|(k, _)| k.clone()).collect();
    Ok(Value::List(keys))
}

pub fn map_values(args: &[Value]) -> Result<Value, PrimitiveError> {
    let map = args.first().and_then(|v| match v { Value::Map(m) => Some(m), _ => None })
        .ok_or_else(|| PrimitiveError("map_values requires map".into()))?;
    let vals: Vec<Value> = map.iter().map(|(_, v)| v.clone()).collect();
    Ok(Value::List(vals))
}

pub fn map_contains(args: &[Value]) -> Result<Value, PrimitiveError> {
    let map = args.first().and_then(|v| match v { Value::Map(m) => Some(m), _ => None })
        .ok_or_else(|| PrimitiveError("map_contains requires map".into()))?;
    let key = args.get(1).cloned().ok_or_else(|| PrimitiveError("map_contains requires key".into()))?;
    Ok(Value::Bool(map.iter().any(|(k, _)| k == &key)))
}

pub fn map_delete(args: &[Value]) -> Result<Value, PrimitiveError> {
    let map = args.first().and_then(|v| match v { Value::Map(m) => Some(m.clone()), _ => None })
        .ok_or_else(|| PrimitiveError("map_delete requires map".into()))?;
    let key = args.get(1).cloned().ok_or_else(|| PrimitiveError("map_delete requires key".into()))?;
    let filtered: Vec<(Value, Value)> = map.into_iter().filter(|(k, _)| k != &key).collect();
    Ok(Value::Map(filtered))
}

pub fn map_merge(args: &[Value]) -> Result<Value, PrimitiveError> {
    let a = args.first().and_then(|v| match v { Value::Map(m) => Some(m.clone()), _ => None })
        .ok_or_else(|| PrimitiveError("map_merge requires map a".into()))?;
    let b = args.get(1).and_then(|v| match v { Value::Map(m) => Some(m.clone()), _ => None })
        .ok_or_else(|| PrimitiveError("map_merge requires map b".into()))?;
    let mut merged = a;
    for (k, v) in b {
        merged.retain(|(mk, _)| mk != &k);
        merged.push((k, v));
    }
    Ok(Value::Map(merged))
}

pub fn map_entries(args: &[Value]) -> Result<Value, PrimitiveError> {
    let map = args.first().and_then(|v| match v { Value::Map(m) => Some(m), _ => None })
        .ok_or_else(|| PrimitiveError("map_entries requires map".into()))?;
    let entries: Vec<Value> = map.iter().map(|(k, v)| {
        Value::List(vec![k.clone(), v.clone()])
    }).collect();
    Ok(Value::List(entries))
}
