// Generated model-specific JSON codecs. Schema declarations are compiled out.
#![allow(unused_mut)]
use super::models::*;
use super::contracts::{ContractError, ContractResult, ErrorReason};
use serde_json::{Value, Map};
fn failure(reason: ErrorReason, id: &str, member: &str, location: &str, detail: impl ToString)->Box<ContractError>{
 Box::new(ContractError{reason,model_id:Some(id.into()),attribute:(!member.is_empty()).then(||member.into()),format:Some("json".into()),rule:None,location:location.into(),klass:None,detail:Some(detail.to_string())})
}
fn object(value:Value,id:&str)->ContractResult<Map<String,Value>> { value.as_object().cloned().ok_or_else(||failure(ErrorReason::InvalidDocument,id,"","/input","expected object")) }
#[allow(dead_code)]
fn empty(v:&Value)->bool { match v { Value::String(v)=>v.is_empty(),Value::Array(v)=>v.is_empty(),Value::Object(v)=>v.is_empty(),_=>false } }
fn numeric_json_err()->&'static str{"numeric JSON is not a portable exact boundary: the portable-exact-json-v1 profile requires exact values as strings; this looks like ruby-native-v1 numeric JSON (Ruby to_json), which is not accepted"}
// Only a plain 64-bit integer (ordered-content indexes) may be a JSON number; any
// fractional, exponent, negative-zero or oversized spelling is not a portable exact boundary.
fn plain_integer(n:&str)->Result<Value,&'static str>{
 let digits=n.strip_prefix('-').unwrap_or(n);
 let ok=!digits.is_empty()&&digits.bytes().all(|c|c.is_ascii_digit())&&n!="-0"&&(n.parse::<i64>().is_ok()||n.parse::<u64>().is_ok());
 if ok{serde_json::from_str::<Value>(n).map_err(|_|numeric_json_err())}else{Err(numeric_json_err())}
}
// RFC 8259 number grammar, nothing else (no sign prefix, spaces, quotes or leading zeros).
#[allow(dead_code)]
fn json_number(s:&str)->bool{
 let b=s.as_bytes();let mut i=usize::from(b.first()==Some(&b'-'));
 let digits=|i:&mut usize|{let at=*i;while *i<b.len()&&b[*i].is_ascii_digit(){*i+=1;}*i-at};
 let whole=i;let n=digits(&mut i);
 if n==0||(n>1&&b[whole]==b'0'){return false;}
 if i<b.len()&&b[i]==b'.'{i+=1;if digits(&mut i)==0{return false;}}
 if i<b.len()&&(b[i]==b'e'||b[i]==b'E'){i+=1;
  if i<b.len()&&(b[i]==b'+'||b[i]==b'-'){i+=1;}
  if digits(&mut i)==0{return false;}}
 i==b.len()
}
// Numbers in a keyed-collection body that no member mapping extracts would be dropped
// unseen; refuse them like every other loose number (`keep` lists the extracted paths).
#[allow(dead_code)]
fn no_loose_except(v:&Value,keep:&[&[&str]],id:&str)->ContractResult<()>{
 if keep.iter().any(|p|p.is_empty()){return Ok(());}
 match v {
  Value::Object(o)=>o.iter().try_for_each(|(k,x)|{let sub:Vec<&[&str]>=keep.iter().filter(|p|p[0]==k.as_str()).map(|p|&p[1..]).collect();no_loose_except(x,&sub,id)}),
  other=>no_loose(other,id),
 }
}
// Bundles with a `hash` attribute parse numbers loosely; every other position must
// still refuse them, so the decoded model is walked once (`strict_*`).
#[allow(dead_code)]
fn no_loose(v:&Value,id:&str)->ContractResult<()>{
 match v {
  Value::Number(n)=>plain_integer(&n.to_string()).map(|_|()).map_err(|e|failure(ErrorReason::InvalidDocument,id,"","/input",e)),
  Value::Array(a)=>a.iter().try_for_each(|x|no_loose(x,id)),
  Value::Object(o)=>o.values().try_for_each(|x|no_loose(x,id)),
  _=>Ok(()),
 }
}
// Inside a hash a number must be exact (the same rule the TS strict reader applies).
#[allow(dead_code)]
fn neutral_numbers(v:&Value,id:&str)->ContractResult<()>{
 match v {
  Value::Number(n)=>NeutralNumber::parse(&n.to_string()).map(|_|()).map_err(|e|failure(ErrorReason::InvalidDocument,id,"","/input",format!("{e}: {n}"))),
  Value::Array(a)=>a.iter().try_for_each(|x|neutral_numbers(x,id)),
  Value::Object(o)=>o.values().try_for_each(|x|neutral_numbers(x,id)),
  _=>Ok(()),
 }
}
#[allow(dead_code)]
fn strict_each(v:&Value,f:&dyn std::ops::Fn(&Value)->ContractResult<()>)->ContractResult<()>{ match v { Value::Array(a)=>a.iter().try_for_each(f), other=>f(other) } }
// A hash keeps its key order only when serde_json keeps insertion order.
#[allow(dead_code)]
fn ordered_maps(id:&str)->ContractResult<()>{
 let mut m=Map::new(); m.insert("b".into(),Value::Null); m.insert("a".into(),Value::Null);
 if m.keys().next().map(String::as_str)==Some("b"){Ok(())}else{Err(failure(ErrorReason::InvalidDocument,id,"","/input","hash attributes need serde_json with the preserve_order feature"))}
}
fn parse_json(input:&str,id:&str)->ContractResult<Value>{
 // Reject duplicate keys before mapping aliases. serde_json::Value alone would
 // silently keep only the last key, erasing an observable input ambiguity.
 struct Unique(Value);
 impl<'de> serde::Deserialize<'de> for Unique {
  fn deserialize<D:serde::Deserializer<'de>>(d:D)->Result<Self,D::Error>{
   struct Visitor;
   impl<'de> serde::de::Visitor<'de> for Visitor {
    type Value=Unique;
    fn expecting(&self,f:&mut std::fmt::Formatter)->std::fmt::Result{f.write_str("JSON without duplicate keys")}
    fn visit_bool<E:serde::de::Error>(self,v:bool)->Result<Unique,E>{Ok(Unique(Value::Bool(v)))}
    // serde_json reads "-0" as the integer 0, so a signed zero is -0 spelled as an integer; keep it as a float
    // that no integer, string or hash position accepts (negative zero is not exact).
    fn visit_i64<E:serde::de::Error>(self,v:i64)->Result<Unique,E>{if NEUTRAL_NUMBERS&&v==0{return Ok(Unique(serde_json::Number::from_f64(-0.0).map_or(Value::Null,Value::Number)));}Ok(Unique(v.into()))}
    fn visit_u64<E:serde::de::Error>(self,v:u64)->Result<Unique,E>{Ok(Unique(v.into()))}
    fn visit_f64<E:serde::de::Error>(self,_:f64)->Result<Unique,E>{Err(E::custom(numeric_json_err()))}
    fn visit_str<E:serde::de::Error>(self,v:&str)->Result<Unique,E>{Ok(Unique(Value::String(v.into())))}
    fn visit_string<E:serde::de::Error>(self,v:String)->Result<Unique,E>{Ok(Unique(Value::String(v)))}
    fn visit_unit<E:serde::de::Error>(self)->Result<Unique,E>{Ok(Unique(Value::Null))}
    fn visit_none<E:serde::de::Error>(self)->Result<Unique,E>{Ok(Unique(Value::Null))}
    fn visit_seq<A:serde::de::SeqAccess<'de>>(self,mut a:A)->Result<Unique,A::Error>{let mut v=Vec::new();while let Some(Unique(x))=a.next_element()?{v.push(x);}Ok(Unique(Value::Array(v)))}
    fn visit_map<A:serde::de::MapAccess<'de>>(self,mut a:A)->Result<Unique,A::Error>{
     let mut v=Map::new();let mut first=true;
     while let Some(k)=a.next_key::<String>()?{
      // With serde_json arbitrary_precision (always on for consumers of this crate)
      // every JSON number arrives as a one-entry map, never through visit_f64.
      // The marker's value must be a lone string in the strict JSON number grammar: nothing
      // else may travel through it (no quoted text, objects, spaces or sibling keys).
      if first && k=="$serde_json::private::Number"{let n:String=a.next_value()?;if a.next_key::<String>()?.is_some(){return Err(serde::de::Error::custom("not a JSON number"));}
      if NEUTRAL_NUMBERS{if !json_number(&n){return Err(serde::de::Error::custom("not a JSON number"));}return serde_json::from_str::<Value>(&n).map(Unique).map_err(serde::de::Error::custom);}
      return plain_integer(&n).map(Unique).map_err(serde::de::Error::custom);}
      first=false;
      let Unique(x)=a.next_value()?;
      if v.insert(k,x).is_some(){return Err(serde::de::Error::custom("duplicate JSON key"));}
     }
     Ok(Unique(Value::Object(v)))
    }
   }
   d.deserialize_any(Visitor)
  }
 }
 serde_json::from_str::<Unique>(input).map(|v|v.0).map_err(|e|failure(ErrorReason::InvalidDocument,id,"","/input",e))
}
#[allow(dead_code)]
fn get_path(v:&Value,path:&[&str])->Option<Value>{let mut at=v;for part in path{at=at.as_object()?.get(*part)?;}Some(at.clone())}
#[allow(dead_code)]
fn put_path(v:&mut Value,path:&[&str],value:Value,id:&str)->ContractResult<()>{
 if path.is_empty(){if !v.is_null(){return Err(failure(ErrorReason::UnsupportedMapping,id,"","/output","overlapping value mappings"));}*v=value;return Ok(());}
 if v.is_null(){*v=Value::Object(Map::new());}
 let map=v.as_object_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,id,"","/output","overlapping mapping paths"))?;
 let slot=map.entry(path[0]).or_insert(Value::Null);put_path(slot,&path[1..],value,id)
}
const NEUTRAL_NUMBERS: bool = false;
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_Args_json(input:&str)->ContractResult<Latexmath_Model_Args>{let data=decode_Latexmath_Model_Args(parse_json(input,"Latexmath::Model::Args@default")?)?; let model:Latexmath_Model_Args=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_Args_json(model:&Latexmath_Model_Args)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_Args(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_Args(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::Args@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "binary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("dividend") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "dividend", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("dividend".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("divisor") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "divisor", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("divisor".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("environment") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "environment", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("environment".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expression") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "expression", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expression".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left_right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "left_right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left_right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("limits") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "limits", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("limits".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("numeric_values") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "numeric_values", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("numeric_values".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("operant") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "operant", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("operant".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("over") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "over", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("over".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "power", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("row_break") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "row_break", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("row_break".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("second_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "second_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("second_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sequence") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "sequence", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sequence".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("subscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "subscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("subscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("substack") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "substack", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("substack".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("supscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "supscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("supscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbols".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table_data") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "table_data", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table_data".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "ternary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("three_per_em_space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "three_per_em_space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("three_per_em_space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_functions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "unary_functions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_functions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("under_over") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "under_over", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("under_over".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("underover") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "underover", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("underover".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unicode_symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "unicode_symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unicode_symbols".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Args@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_Args(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_Args(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Latexmath_Model_Base(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("binary"){
if !v.is_null() {
v=decode_Latexmath_Model_Binary(v)?;
}
data.insert("binary".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("dividend"){
if !v.is_null() {
v=decode_Latexmath_Model_Dividend(v)?;
}
data.insert("dividend".into(),v);}
if let Some(mut v)=data.remove("divisor"){
if !v.is_null() {
v=decode_Latexmath_Model_Divisor(v)?;
}
data.insert("divisor".into(),v);}
if let Some(mut v)=data.remove("environment"){
if !v.is_null() {
v=decode_Latexmath_Model_Environment(v)?;
}
data.insert("environment".into(),v);}
if let Some(mut v)=data.remove("expression"){
if !v.is_null() {
v=decode_Latexmath_Model_Expression(v)?;
}
data.insert("expression".into(),v);}
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Latexmath_Model_FirstValue(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("fonts"){
data.insert("fonts".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Latexmath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("left_right"){
if !v.is_null() {
v=decode_Latexmath_Model_LeftRight(v)?;
}
data.insert("left_right".into(),v);}
if let Some(mut v)=data.remove("limits"){
if !v.is_null() {
v=decode_Latexmath_Model_Limits(v)?;
}
data.insert("limits".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("numeric_values"){
data.insert("numeric_values".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("operant"){
data.insert("operant".into(),v);}
if let Some(mut v)=data.remove("over"){
if !v.is_null() {
v=decode_Latexmath_Model_Over(v)?;
}
data.insert("over".into(),v);}
if let Some(mut v)=data.remove("power"){
if !v.is_null() {
v=decode_Latexmath_Model_Power(v)?;
}
data.insert("power".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Latexmath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("row_break"){
data.insert("row_break".into(),v);}
if let Some(mut v)=data.remove("second_value"){
if !v.is_null() {
v=decode_Latexmath_Model_SecondValue(v)?;
}
data.insert("second_value".into(),v);}
if let Some(mut v)=data.remove("sequence"){
if !v.is_null() {
v=decode_Latexmath_Model_Sequence(v)?;
}
data.insert("sequence".into(),v);}
if let Some(mut v)=data.remove("space"){
data.insert("space".into(),v);}
if let Some(mut v)=data.remove("subscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Subscript(v)?;
}
data.insert("subscript".into(),v);}
if let Some(mut v)=data.remove("substack"){
data.insert("substack".into(),v);}
if let Some(mut v)=data.remove("supscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Supscript(v)?;
}
data.insert("supscript".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Latexmath_Model_SymbolNode(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("symbols".into(),v);}
if let Some(mut v)=data.remove("table_data"){
if !v.is_null() {
v=decode_Latexmath_Model_TableData(v)?;
}
data.insert("table_data".into(),v);}
if let Some(mut v)=data.remove("ternary"){
data.insert("ternary".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
if !v.is_null() {
v=decode_Latexmath_Model_TernaryClass(v)?;
}
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("three_per_em_space"){
data.insert("three_per_em_space".into(),v);}
if let Some(mut v)=data.remove("unary"){
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("unary_functions"){
if !v.is_null() {
v=decode_Latexmath_Model_UnaryFunctions(v)?;
}
data.insert("unary_functions".into(),v);}
if let Some(mut v)=data.remove("under_over"){
if !v.is_null() {
v=decode_Latexmath_Model_UnderOver(v)?;
}
data.insert("under_over".into(),v);}
if let Some(mut v)=data.remove("underover"){
data.insert("underover".into(),v);}
if let Some(mut v)=data.remove("unicode_symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("unicode_symbols".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_Args(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::Args@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Base(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("binary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Binary(v)?;
}
wire.insert("binary".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("dividend").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Dividend(v)?;
}
wire.insert("dividend".into(),v);}
for mut v in data.remove("divisor").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Divisor(v)?;
}
wire.insert("divisor".into(),v);}
for mut v in data.remove("environment").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Environment(v)?;
}
wire.insert("environment".into(),v);}
for mut v in data.remove("expression").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Expression(v)?;
}
wire.insert("expression".into(),v);}
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_FirstValue(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("left_right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_LeftRight(v)?;
}
wire.insert("left_right".into(),v);}
for mut v in data.remove("limits").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Limits(v)?;
}
wire.insert("limits".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("numeric_values").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("numeric_values".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("operant").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("operant".into(),v);}
for mut v in data.remove("over").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Over(v)?;
}
wire.insert("over".into(),v);}
for mut v in data.remove("power").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Power(v)?;
}
wire.insert("power".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("row_break").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("row_break".into(),v);}
for mut v in data.remove("second_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SecondValue(v)?;
}
wire.insert("second_value".into(),v);}
for mut v in data.remove("sequence").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Sequence(v)?;
}
wire.insert("sequence".into(),v);}
for mut v in data.remove("space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("space".into(),v);}
for mut v in data.remove("subscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Subscript(v)?;
}
wire.insert("subscript".into(),v);}
for mut v in data.remove("substack").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("substack".into(),v);}
for mut v in data.remove("supscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Supscript(v)?;
}
wire.insert("supscript".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SymbolNode(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("symbols".into(),v);}
for mut v in data.remove("table_data").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TableData(v)?;
}
wire.insert("table_data".into(),v);}
for mut v in data.remove("ternary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TernaryClass(v)?;
}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("text".into(),v);}
for mut v in data.remove("three_per_em_space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("three_per_em_space".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("unary".into(),v);}
for mut v in data.remove("unary_functions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnaryFunctions(v)?;
}
wire.insert("unary_functions".into(),v);}
for mut v in data.remove("under_over").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnderOver(v)?;
}
wire.insert("under_over".into(),v);}
for mut v in data.remove("underover").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("underover".into(),v);}
for mut v in data.remove("unicode_symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("unicode_symbols".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_Base_json(input:&str)->ContractResult<Latexmath_Model_Base>{let data=decode_Latexmath_Model_Base(parse_json(input,"Latexmath::Model::Base@default")?)?; let model:Latexmath_Model_Base=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_Base_json(model:&Latexmath_Model_Base)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_Base(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_Base(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::Base@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("binary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "binary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_functions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "binary_functions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_functions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("environment") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "environment", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("environment".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expression") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "expression", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expression".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left_right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "left_right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left_right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("numeric_values") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "numeric_values", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("numeric_values".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("operant") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "operant", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("operant".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("row_break") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "row_break", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("row_break".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("subscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "subscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("subscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("supscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "supscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("supscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbols".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "ternary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("three_per_em_space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "three_per_em_space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("three_per_em_space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_functions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "unary_functions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_functions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("underover") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "underover", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("underover".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unicode_symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "unicode_symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unicode_symbols".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Base@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_Base(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_Base(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("binary"){
if !v.is_null() {
v=decode_Latexmath_Model_Binary(v)?;
}
data.insert("binary".into(),v);}
if let Some(mut v)=data.remove("binary_functions"){
if !v.is_null() {
v=decode_Latexmath_Model_BinaryFunctions(v)?;
}
data.insert("binary_functions".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("environment"){
if !v.is_null() {
v=decode_Latexmath_Model_Environment(v)?;
}
data.insert("environment".into(),v);}
if let Some(mut v)=data.remove("expression"){
if !v.is_null() {
v=decode_Latexmath_Model_Expression(v)?;
}
data.insert("expression".into(),v);}
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Latexmath_Model_FirstValue(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("fonts"){
data.insert("fonts".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Latexmath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("left_right"){
if !v.is_null() {
v=decode_Latexmath_Model_LeftRight(v)?;
}
data.insert("left_right".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("numeric_values"){
data.insert("numeric_values".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("operant"){
data.insert("operant".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Latexmath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("row_break"){
data.insert("row_break".into(),v);}
if let Some(mut v)=data.remove("space"){
data.insert("space".into(),v);}
if let Some(mut v)=data.remove("subscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Subscript(v)?;
}
data.insert("subscript".into(),v);}
if let Some(mut v)=data.remove("supscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Supscript(v)?;
}
data.insert("supscript".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Latexmath_Model_SymbolNode(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("symbols".into(),v);}
if let Some(mut v)=data.remove("ternary"){
data.insert("ternary".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
if !v.is_null() {
v=decode_Latexmath_Model_TernaryClass(v)?;
}
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("three_per_em_space"){
data.insert("three_per_em_space".into(),v);}
if let Some(mut v)=data.remove("unary"){
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("unary_functions"){
if !v.is_null() {
v=decode_Latexmath_Model_UnaryFunctions(v)?;
}
data.insert("unary_functions".into(),v);}
if let Some(mut v)=data.remove("underover"){
data.insert("underover".into(),v);}
if let Some(mut v)=data.remove("unicode_symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("unicode_symbols".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_Base(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::Base@default")?;let mut wire=Map::new();
for mut v in data.remove("binary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Binary(v)?;
}
wire.insert("binary".into(),v);}
for mut v in data.remove("binary_functions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_BinaryFunctions(v)?;
}
wire.insert("binary_functions".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("environment").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Environment(v)?;
}
wire.insert("environment".into(),v);}
for mut v in data.remove("expression").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Expression(v)?;
}
wire.insert("expression".into(),v);}
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_FirstValue(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("left_right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_LeftRight(v)?;
}
wire.insert("left_right".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("numeric_values").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("numeric_values".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("operant").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("operant".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("row_break").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("row_break".into(),v);}
for mut v in data.remove("space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("space".into(),v);}
for mut v in data.remove("subscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Subscript(v)?;
}
wire.insert("subscript".into(),v);}
for mut v in data.remove("supscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Supscript(v)?;
}
wire.insert("supscript".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SymbolNode(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("symbols".into(),v);}
for mut v in data.remove("ternary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TernaryClass(v)?;
}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("text".into(),v);}
for mut v in data.remove("three_per_em_space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("three_per_em_space".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("unary".into(),v);}
for mut v in data.remove("unary_functions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnaryFunctions(v)?;
}
wire.insert("unary_functions".into(),v);}
for mut v in data.remove("underover").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("underover".into(),v);}
for mut v in data.remove("unicode_symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("unicode_symbols".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_Binary_json(input:&str)->ContractResult<Latexmath_Model_Binary>{let data=decode_Latexmath_Model_Binary(parse_json(input,"Latexmath::Model::Binary@default")?)?; let model:Latexmath_Model_Binary=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Binary@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_Binary_json(model:&Latexmath_Model_Binary)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Binary@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_Binary(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Binary@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_Binary(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::Binary@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("binary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Binary@default", "binary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Binary@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Binary@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("literal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Binary@default", "literal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("root") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Binary@default", "root", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("root".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("rule") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Binary@default", "rule", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("rule".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("second_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Binary@default", "second_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("second_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sqrt") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Binary@default", "sqrt", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sqrt".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("third_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Binary@default", "third_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("third_value".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Binary@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_Binary(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_Binary(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("binary"){
if !v.is_null() {
v=decode_Latexmath_Model_Binary(v)?;
}
data.insert("binary".into(),v);}
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Latexmath_Model_FirstValue(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Latexmath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("literal"){
data.insert("literal".into(),v);}
if let Some(mut v)=data.remove("root"){
data.insert("root".into(),v);}
if let Some(mut v)=data.remove("rule"){
data.insert("rule".into(),v);}
if let Some(mut v)=data.remove("second_value"){
if !v.is_null() {
v=decode_Latexmath_Model_SecondValue(v)?;
}
data.insert("second_value".into(),v);}
if let Some(mut v)=data.remove("sqrt"){
data.insert("sqrt".into(),v);}
if let Some(mut v)=data.remove("third_value"){
if !v.is_null() {
v=decode_Latexmath_Model_ThirdValue(v)?;
}
data.insert("third_value".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_Binary(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::Binary@default")?;let mut wire=Map::new();
for mut v in data.remove("binary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Binary(v)?;
}
wire.insert("binary".into(),v);}
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_FirstValue(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("literal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal".into(),v);}
for mut v in data.remove("root").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("root".into(),v);}
for mut v in data.remove("rule").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("rule".into(),v);}
for mut v in data.remove("second_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SecondValue(v)?;
}
wire.insert("second_value".into(),v);}
for mut v in data.remove("sqrt").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("sqrt".into(),v);}
for mut v in data.remove("third_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_ThirdValue(v)?;
}
wire.insert("third_value".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_BinaryFunctions_json(input:&str)->ContractResult<Latexmath_Model_BinaryFunctions>{let data=decode_Latexmath_Model_BinaryFunctions(parse_json(input,"Latexmath::Model::BinaryFunctions@default")?)?; let model:Latexmath_Model_BinaryFunctions=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::BinaryFunctions@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_BinaryFunctions_json(model:&Latexmath_Model_BinaryFunctions)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::BinaryFunctions@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_BinaryFunctions(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::BinaryFunctions@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_BinaryFunctions(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::BinaryFunctions@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("binary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::BinaryFunctions@default", "binary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::BinaryFunctions@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("second_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::BinaryFunctions@default", "second_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("second_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("under_over") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::BinaryFunctions@default", "under_over", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("under_over".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::BinaryFunctions@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_BinaryFunctions(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_BinaryFunctions(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("binary"){
if !v.is_null() {
v=decode_Latexmath_Model_Binary(v)?;
}
data.insert("binary".into(),v);}
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Latexmath_Model_FirstValue(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("second_value"){
if !v.is_null() {
v=decode_Latexmath_Model_SecondValue(v)?;
}
data.insert("second_value".into(),v);}
if let Some(mut v)=data.remove("under_over"){
if !v.is_null() {
v=decode_Latexmath_Model_UnderOver(v)?;
}
data.insert("under_over".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_BinaryFunctions(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::BinaryFunctions@default")?;let mut wire=Map::new();
for mut v in data.remove("binary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Binary(v)?;
}
wire.insert("binary".into(),v);}
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_FirstValue(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("second_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SecondValue(v)?;
}
wire.insert("second_value".into(),v);}
for mut v in data.remove("under_over").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnderOver(v)?;
}
wire.insert("under_over".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_Dividend_json(input:&str)->ContractResult<Latexmath_Model_Dividend>{let data=decode_Latexmath_Model_Dividend(parse_json(input,"Latexmath::Model::Dividend@default")?)?; let model:Latexmath_Model_Dividend=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_Dividend_json(model:&Latexmath_Model_Dividend)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_Dividend(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_Dividend(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::Dividend@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "binary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("dividend") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "dividend", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("dividend".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("divisor") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "divisor", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("divisor".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("environment") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "environment", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("environment".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expression") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "expression", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expression".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("items") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "items", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("items".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left_right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "left_right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left_right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("limits") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "limits", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("limits".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("numeric_values") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "numeric_values", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("numeric_values".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("operant") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "operant", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("operant".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("over") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "over", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("over".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "power", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("row_break") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "row_break", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("row_break".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("second_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "second_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("second_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sequence") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "sequence", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sequence".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("subscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "subscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("subscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("substack") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "substack", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("substack".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("supscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "supscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("supscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbols".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table_data") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "table_data", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table_data".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "ternary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("three_per_em_space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "three_per_em_space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("three_per_em_space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_functions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "unary_functions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_functions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("under_over") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "under_over", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("under_over".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("underover") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "underover", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("underover".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unicode_symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "unicode_symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unicode_symbols".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_Dividend(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_Dividend(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Latexmath_Model_Base(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("binary"){
if !v.is_null() {
v=decode_Latexmath_Model_Binary(v)?;
}
data.insert("binary".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("dividend"){
if !v.is_null() {
v=decode_Latexmath_Model_Dividend(v)?;
}
data.insert("dividend".into(),v);}
if let Some(mut v)=data.remove("divisor"){
if !v.is_null() {
v=decode_Latexmath_Model_Divisor(v)?;
}
data.insert("divisor".into(),v);}
if let Some(mut v)=data.remove("environment"){
if !v.is_null() {
v=decode_Latexmath_Model_Environment(v)?;
}
data.insert("environment".into(),v);}
if let Some(mut v)=data.remove("expression"){
if !v.is_null() {
v=decode_Latexmath_Model_Expression(v)?;
}
data.insert("expression".into(),v);}
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Latexmath_Model_FirstValue(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("fonts"){
data.insert("fonts".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Latexmath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("items"){
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "items", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=decode_Latexmath_Model_ItemsItem(v)?;
*slot=v; }
}
data.insert("items".into(),v);}
if let Some(mut v)=data.remove("left_right"){
if !v.is_null() {
v=decode_Latexmath_Model_LeftRight(v)?;
}
data.insert("left_right".into(),v);}
if let Some(mut v)=data.remove("limits"){
if !v.is_null() {
v=decode_Latexmath_Model_Limits(v)?;
}
data.insert("limits".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("numeric_values"){
data.insert("numeric_values".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("operant"){
data.insert("operant".into(),v);}
if let Some(mut v)=data.remove("over"){
if !v.is_null() {
v=decode_Latexmath_Model_Over(v)?;
}
data.insert("over".into(),v);}
if let Some(mut v)=data.remove("power"){
if !v.is_null() {
v=decode_Latexmath_Model_Power(v)?;
}
data.insert("power".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Latexmath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("row_break"){
data.insert("row_break".into(),v);}
if let Some(mut v)=data.remove("second_value"){
if !v.is_null() {
v=decode_Latexmath_Model_SecondValue(v)?;
}
data.insert("second_value".into(),v);}
if let Some(mut v)=data.remove("sequence"){
if !v.is_null() {
v=decode_Latexmath_Model_Sequence(v)?;
}
data.insert("sequence".into(),v);}
if let Some(mut v)=data.remove("space"){
data.insert("space".into(),v);}
if let Some(mut v)=data.remove("subscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Subscript(v)?;
}
data.insert("subscript".into(),v);}
if let Some(mut v)=data.remove("substack"){
data.insert("substack".into(),v);}
if let Some(mut v)=data.remove("supscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Supscript(v)?;
}
data.insert("supscript".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Latexmath_Model_SymbolNode(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("symbols".into(),v);}
if let Some(mut v)=data.remove("table_data"){
if !v.is_null() {
v=decode_Latexmath_Model_TableData(v)?;
}
data.insert("table_data".into(),v);}
if let Some(mut v)=data.remove("ternary"){
data.insert("ternary".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
if !v.is_null() {
v=decode_Latexmath_Model_TernaryClass(v)?;
}
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("three_per_em_space"){
data.insert("three_per_em_space".into(),v);}
if let Some(mut v)=data.remove("unary"){
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("unary_functions"){
if !v.is_null() {
v=decode_Latexmath_Model_UnaryFunctions(v)?;
}
data.insert("unary_functions".into(),v);}
if let Some(mut v)=data.remove("under_over"){
if !v.is_null() {
v=decode_Latexmath_Model_UnderOver(v)?;
}
data.insert("under_over".into(),v);}
if let Some(mut v)=data.remove("underover"){
data.insert("underover".into(),v);}
if let Some(mut v)=data.remove("unicode_symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("unicode_symbols".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_Dividend(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::Dividend@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Base(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("binary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Binary(v)?;
}
wire.insert("binary".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("dividend").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Dividend(v)?;
}
wire.insert("dividend".into(),v);}
for mut v in data.remove("divisor").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Divisor(v)?;
}
wire.insert("divisor".into(),v);}
for mut v in data.remove("environment").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Environment(v)?;
}
wire.insert("environment".into(),v);}
for mut v in data.remove("expression").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Expression(v)?;
}
wire.insert("expression".into(),v);}
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_FirstValue(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("items").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Latexmath::Model::Dividend@default", "items", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=encode_Latexmath_Model_ItemsItem(v)?;
*slot=v; }
}
wire.insert("items".into(),v);}
for mut v in data.remove("left_right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_LeftRight(v)?;
}
wire.insert("left_right".into(),v);}
for mut v in data.remove("limits").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Limits(v)?;
}
wire.insert("limits".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("numeric_values").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("numeric_values".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("operant").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("operant".into(),v);}
for mut v in data.remove("over").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Over(v)?;
}
wire.insert("over".into(),v);}
for mut v in data.remove("power").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Power(v)?;
}
wire.insert("power".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("row_break").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("row_break".into(),v);}
for mut v in data.remove("second_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SecondValue(v)?;
}
wire.insert("second_value".into(),v);}
for mut v in data.remove("sequence").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Sequence(v)?;
}
wire.insert("sequence".into(),v);}
for mut v in data.remove("space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("space".into(),v);}
for mut v in data.remove("subscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Subscript(v)?;
}
wire.insert("subscript".into(),v);}
for mut v in data.remove("substack").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("substack".into(),v);}
for mut v in data.remove("supscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Supscript(v)?;
}
wire.insert("supscript".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SymbolNode(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("symbols".into(),v);}
for mut v in data.remove("table_data").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TableData(v)?;
}
wire.insert("table_data".into(),v);}
for mut v in data.remove("ternary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TernaryClass(v)?;
}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("text".into(),v);}
for mut v in data.remove("three_per_em_space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("three_per_em_space".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("unary".into(),v);}
for mut v in data.remove("unary_functions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnaryFunctions(v)?;
}
wire.insert("unary_functions".into(),v);}
for mut v in data.remove("under_over").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnderOver(v)?;
}
wire.insert("under_over".into(),v);}
for mut v in data.remove("underover").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("underover".into(),v);}
for mut v in data.remove("unicode_symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("unicode_symbols".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_Divisor_json(input:&str)->ContractResult<Latexmath_Model_Divisor>{let data=decode_Latexmath_Model_Divisor(parse_json(input,"Latexmath::Model::Divisor@default")?)?; let model:Latexmath_Model_Divisor=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_Divisor_json(model:&Latexmath_Model_Divisor)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_Divisor(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_Divisor(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::Divisor@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "binary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("dividend") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "dividend", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("dividend".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("divisor") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "divisor", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("divisor".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("environment") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "environment", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("environment".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expression") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "expression", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expression".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("items") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "items", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("items".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left_right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "left_right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left_right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("limits") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "limits", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("limits".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("numeric_values") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "numeric_values", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("numeric_values".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("operant") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "operant", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("operant".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("over") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "over", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("over".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "power", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("row_break") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "row_break", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("row_break".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("second_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "second_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("second_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sequence") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "sequence", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sequence".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("subscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "subscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("subscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("substack") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "substack", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("substack".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("supscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "supscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("supscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbols".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table_data") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "table_data", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table_data".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "ternary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("three_per_em_space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "three_per_em_space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("three_per_em_space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_functions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "unary_functions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_functions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("under_over") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "under_over", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("under_over".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("underover") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "underover", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("underover".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unicode_symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "unicode_symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unicode_symbols".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_Divisor(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_Divisor(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Latexmath_Model_Base(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("binary"){
if !v.is_null() {
v=decode_Latexmath_Model_Binary(v)?;
}
data.insert("binary".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("dividend"){
if !v.is_null() {
v=decode_Latexmath_Model_Dividend(v)?;
}
data.insert("dividend".into(),v);}
if let Some(mut v)=data.remove("divisor"){
if !v.is_null() {
v=decode_Latexmath_Model_Divisor(v)?;
}
data.insert("divisor".into(),v);}
if let Some(mut v)=data.remove("environment"){
if !v.is_null() {
v=decode_Latexmath_Model_Environment(v)?;
}
data.insert("environment".into(),v);}
if let Some(mut v)=data.remove("expression"){
if !v.is_null() {
v=decode_Latexmath_Model_Expression(v)?;
}
data.insert("expression".into(),v);}
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Latexmath_Model_FirstValue(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("fonts"){
data.insert("fonts".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Latexmath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("items"){
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "items", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=decode_Latexmath_Model_ItemsItem(v)?;
*slot=v; }
}
data.insert("items".into(),v);}
if let Some(mut v)=data.remove("left_right"){
if !v.is_null() {
v=decode_Latexmath_Model_LeftRight(v)?;
}
data.insert("left_right".into(),v);}
if let Some(mut v)=data.remove("limits"){
if !v.is_null() {
v=decode_Latexmath_Model_Limits(v)?;
}
data.insert("limits".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("numeric_values"){
data.insert("numeric_values".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("operant"){
data.insert("operant".into(),v);}
if let Some(mut v)=data.remove("over"){
if !v.is_null() {
v=decode_Latexmath_Model_Over(v)?;
}
data.insert("over".into(),v);}
if let Some(mut v)=data.remove("power"){
if !v.is_null() {
v=decode_Latexmath_Model_Power(v)?;
}
data.insert("power".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Latexmath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("row_break"){
data.insert("row_break".into(),v);}
if let Some(mut v)=data.remove("second_value"){
if !v.is_null() {
v=decode_Latexmath_Model_SecondValue(v)?;
}
data.insert("second_value".into(),v);}
if let Some(mut v)=data.remove("sequence"){
if !v.is_null() {
v=decode_Latexmath_Model_Sequence(v)?;
}
data.insert("sequence".into(),v);}
if let Some(mut v)=data.remove("space"){
data.insert("space".into(),v);}
if let Some(mut v)=data.remove("subscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Subscript(v)?;
}
data.insert("subscript".into(),v);}
if let Some(mut v)=data.remove("substack"){
data.insert("substack".into(),v);}
if let Some(mut v)=data.remove("supscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Supscript(v)?;
}
data.insert("supscript".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Latexmath_Model_SymbolNode(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("symbols".into(),v);}
if let Some(mut v)=data.remove("table_data"){
if !v.is_null() {
v=decode_Latexmath_Model_TableData(v)?;
}
data.insert("table_data".into(),v);}
if let Some(mut v)=data.remove("ternary"){
data.insert("ternary".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
if !v.is_null() {
v=decode_Latexmath_Model_TernaryClass(v)?;
}
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("three_per_em_space"){
data.insert("three_per_em_space".into(),v);}
if let Some(mut v)=data.remove("unary"){
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("unary_functions"){
if !v.is_null() {
v=decode_Latexmath_Model_UnaryFunctions(v)?;
}
data.insert("unary_functions".into(),v);}
if let Some(mut v)=data.remove("under_over"){
if !v.is_null() {
v=decode_Latexmath_Model_UnderOver(v)?;
}
data.insert("under_over".into(),v);}
if let Some(mut v)=data.remove("underover"){
data.insert("underover".into(),v);}
if let Some(mut v)=data.remove("unicode_symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("unicode_symbols".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_Divisor(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::Divisor@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Base(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("binary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Binary(v)?;
}
wire.insert("binary".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("dividend").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Dividend(v)?;
}
wire.insert("dividend".into(),v);}
for mut v in data.remove("divisor").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Divisor(v)?;
}
wire.insert("divisor".into(),v);}
for mut v in data.remove("environment").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Environment(v)?;
}
wire.insert("environment".into(),v);}
for mut v in data.remove("expression").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Expression(v)?;
}
wire.insert("expression".into(),v);}
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_FirstValue(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("items").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Latexmath::Model::Divisor@default", "items", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=encode_Latexmath_Model_ItemsItem(v)?;
*slot=v; }
}
wire.insert("items".into(),v);}
for mut v in data.remove("left_right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_LeftRight(v)?;
}
wire.insert("left_right".into(),v);}
for mut v in data.remove("limits").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Limits(v)?;
}
wire.insert("limits".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("numeric_values").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("numeric_values".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("operant").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("operant".into(),v);}
for mut v in data.remove("over").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Over(v)?;
}
wire.insert("over".into(),v);}
for mut v in data.remove("power").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Power(v)?;
}
wire.insert("power".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("row_break").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("row_break".into(),v);}
for mut v in data.remove("second_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SecondValue(v)?;
}
wire.insert("second_value".into(),v);}
for mut v in data.remove("sequence").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Sequence(v)?;
}
wire.insert("sequence".into(),v);}
for mut v in data.remove("space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("space".into(),v);}
for mut v in data.remove("subscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Subscript(v)?;
}
wire.insert("subscript".into(),v);}
for mut v in data.remove("substack").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("substack".into(),v);}
for mut v in data.remove("supscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Supscript(v)?;
}
wire.insert("supscript".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SymbolNode(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("symbols".into(),v);}
for mut v in data.remove("table_data").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TableData(v)?;
}
wire.insert("table_data".into(),v);}
for mut v in data.remove("ternary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TernaryClass(v)?;
}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("text".into(),v);}
for mut v in data.remove("three_per_em_space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("three_per_em_space".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("unary".into(),v);}
for mut v in data.remove("unary_functions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnaryFunctions(v)?;
}
wire.insert("unary_functions".into(),v);}
for mut v in data.remove("under_over").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnderOver(v)?;
}
wire.insert("under_over".into(),v);}
for mut v in data.remove("underover").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("underover".into(),v);}
for mut v in data.remove("unicode_symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("unicode_symbols".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_Document_json(input:&str)->ContractResult<Latexmath_Model_Document>{let data=decode_Latexmath_Model_Document(parse_json(input,"Latexmath::Model::Document@default")?)?; let model:Latexmath_Model_Document=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Document@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_Document_json(model:&Latexmath_Model_Document)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Document@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_Document(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Document@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_Document(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::Document@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("expression") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Document@default", "expression", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expression".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("decimal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Document@default", "decimal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("decimal".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Document@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_Document(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_Document(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("expression"){
if !v.is_null() {
v=decode_Latexmath_Model_LatexTree(v)?;
}
data.insert("expression".into(),v);}
if let Some(mut v)=data.remove("decimal"){
data.insert("decimal".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_Document(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::Document@default")?;let mut wire=Map::new();
for mut v in data.remove("expression").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_LatexTree(v)?;
}
wire.insert("expression".into(),v);}
for mut v in data.remove("decimal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("decimal".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_Ending_json(input:&str)->ContractResult<Latexmath_Model_Ending>{let data=decode_Latexmath_Model_Ending(parse_json(input,"Latexmath::Model::Ending@default")?)?; let model:Latexmath_Model_Ending=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Ending@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_Ending_json(model:&Latexmath_Model_Ending)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Ending@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_Ending(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Ending@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_Ending(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::Ending@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("binary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Ending@default", "binary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Ending@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("environment") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Ending@default", "environment", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("environment".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Ending@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Ending@default", "fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Ending@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Ending@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Ending@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("numeric_values") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Ending@default", "numeric_values", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("numeric_values".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Ending@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("operant") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Ending@default", "operant", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("operant".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Ending@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("row_break") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Ending@default", "row_break", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("row_break".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Ending@default", "space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("subscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Ending@default", "subscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("subscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("supscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Ending@default", "supscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("supscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Ending@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Ending@default", "symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbols".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Ending@default", "ternary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Ending@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Ending@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("three_per_em_space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Ending@default", "three_per_em_space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("three_per_em_space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Ending@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_functions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Ending@default", "unary_functions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_functions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("underover") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Ending@default", "underover", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("underover".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unicode_symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Ending@default", "unicode_symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unicode_symbols".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Ending@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_Ending(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_Ending(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("binary"){
if !v.is_null() {
v=decode_Latexmath_Model_Binary(v)?;
}
data.insert("binary".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("environment"){
if !v.is_null() {
v=decode_Latexmath_Model_Environment(v)?;
}
data.insert("environment".into(),v);}
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Latexmath_Model_FirstValue(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("fonts"){
data.insert("fonts".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Latexmath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("numeric_values"){
data.insert("numeric_values".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("operant"){
data.insert("operant".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Latexmath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("row_break"){
data.insert("row_break".into(),v);}
if let Some(mut v)=data.remove("space"){
data.insert("space".into(),v);}
if let Some(mut v)=data.remove("subscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Subscript(v)?;
}
data.insert("subscript".into(),v);}
if let Some(mut v)=data.remove("supscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Supscript(v)?;
}
data.insert("supscript".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Latexmath_Model_SymbolNode(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("symbols".into(),v);}
if let Some(mut v)=data.remove("ternary"){
data.insert("ternary".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
if !v.is_null() {
v=decode_Latexmath_Model_TernaryClass(v)?;
}
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("three_per_em_space"){
data.insert("three_per_em_space".into(),v);}
if let Some(mut v)=data.remove("unary"){
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("unary_functions"){
if !v.is_null() {
v=decode_Latexmath_Model_UnaryFunctions(v)?;
}
data.insert("unary_functions".into(),v);}
if let Some(mut v)=data.remove("underover"){
data.insert("underover".into(),v);}
if let Some(mut v)=data.remove("unicode_symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("unicode_symbols".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_Ending(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::Ending@default")?;let mut wire=Map::new();
for mut v in data.remove("binary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Binary(v)?;
}
wire.insert("binary".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("environment").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Environment(v)?;
}
wire.insert("environment".into(),v);}
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_FirstValue(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("numeric_values").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("numeric_values".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("operant").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("operant".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("row_break").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("row_break".into(),v);}
for mut v in data.remove("space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("space".into(),v);}
for mut v in data.remove("subscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Subscript(v)?;
}
wire.insert("subscript".into(),v);}
for mut v in data.remove("supscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Supscript(v)?;
}
wire.insert("supscript".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SymbolNode(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("symbols".into(),v);}
for mut v in data.remove("ternary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TernaryClass(v)?;
}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("text".into(),v);}
for mut v in data.remove("three_per_em_space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("three_per_em_space".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("unary".into(),v);}
for mut v in data.remove("unary_functions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnaryFunctions(v)?;
}
wire.insert("unary_functions".into(),v);}
for mut v in data.remove("underover").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("underover".into(),v);}
for mut v in data.remove("unicode_symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("unicode_symbols".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_Environment_json(input:&str)->ContractResult<Latexmath_Model_Environment>{let data=decode_Latexmath_Model_Environment(parse_json(input,"Latexmath::Model::Environment@default")?)?; let model:Latexmath_Model_Environment=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_Environment_json(model:&Latexmath_Model_Environment)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_Environment(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_Environment(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::Environment@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("args") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "args", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("args".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("asterisk") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "asterisk", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("asterisk".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "binary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ending") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "ending", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ending".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("environment") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "environment", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("environment".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("literal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "literal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("numeric_values") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "numeric_values", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("numeric_values".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("operant") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "operant", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("operant".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("options") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "options", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("options".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("row_break") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "row_break", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("row_break".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("subscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "subscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("subscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("supscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "supscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("supscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbols".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table_data") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "table_data", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table_data".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "ternary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("three_per_em_space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "three_per_em_space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("three_per_em_space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_functions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "unary_functions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_functions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("underover") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "underover", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("underover".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unicode_symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "unicode_symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unicode_symbols".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Environment@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_Environment(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_Environment(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("args"){
if !v.is_null() {
v=decode_Latexmath_Model_Args(v)?;
}
data.insert("args".into(),v);}
if let Some(mut v)=data.remove("asterisk"){
data.insert("asterisk".into(),v);}
if let Some(mut v)=data.remove("binary"){
if !v.is_null() {
v=decode_Latexmath_Model_Binary(v)?;
}
data.insert("binary".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("ending"){
if !v.is_null() {
v=decode_Latexmath_Model_Ending(v)?;
}
data.insert("ending".into(),v);}
if let Some(mut v)=data.remove("environment"){
if !v.is_null() {
v=decode_Latexmath_Model_Environment(v)?;
}
data.insert("environment".into(),v);}
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Latexmath_Model_FirstValue(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("fonts"){
data.insert("fonts".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Latexmath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("literal"){
data.insert("literal".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("numeric_values"){
data.insert("numeric_values".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("operant"){
data.insert("operant".into(),v);}
if let Some(mut v)=data.remove("options"){
if !v.is_null() {
v=decode_Latexmath_Model_Options(v)?;
}
data.insert("options".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Latexmath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("row_break"){
data.insert("row_break".into(),v);}
if let Some(mut v)=data.remove("space"){
data.insert("space".into(),v);}
if let Some(mut v)=data.remove("subscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Subscript(v)?;
}
data.insert("subscript".into(),v);}
if let Some(mut v)=data.remove("supscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Supscript(v)?;
}
data.insert("supscript".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Latexmath_Model_SymbolNode(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("symbols".into(),v);}
if let Some(mut v)=data.remove("table_data"){
if !v.is_null() {
v=decode_Latexmath_Model_TableData(v)?;
}
data.insert("table_data".into(),v);}
if let Some(mut v)=data.remove("ternary"){
data.insert("ternary".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
if !v.is_null() {
v=decode_Latexmath_Model_TernaryClass(v)?;
}
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("three_per_em_space"){
data.insert("three_per_em_space".into(),v);}
if let Some(mut v)=data.remove("unary"){
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("unary_functions"){
if !v.is_null() {
v=decode_Latexmath_Model_UnaryFunctions(v)?;
}
data.insert("unary_functions".into(),v);}
if let Some(mut v)=data.remove("underover"){
data.insert("underover".into(),v);}
if let Some(mut v)=data.remove("unicode_symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("unicode_symbols".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_Environment(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::Environment@default")?;let mut wire=Map::new();
for mut v in data.remove("args").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Args(v)?;
}
wire.insert("args".into(),v);}
for mut v in data.remove("asterisk").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("asterisk".into(),v);}
for mut v in data.remove("binary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Binary(v)?;
}
wire.insert("binary".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("ending").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Ending(v)?;
}
wire.insert("ending".into(),v);}
for mut v in data.remove("environment").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Environment(v)?;
}
wire.insert("environment".into(),v);}
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_FirstValue(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("literal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("numeric_values").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("numeric_values".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("operant").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("operant".into(),v);}
for mut v in data.remove("options").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Options(v)?;
}
wire.insert("options".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("row_break").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("row_break".into(),v);}
for mut v in data.remove("space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("space".into(),v);}
for mut v in data.remove("subscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Subscript(v)?;
}
wire.insert("subscript".into(),v);}
for mut v in data.remove("supscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Supscript(v)?;
}
wire.insert("supscript".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SymbolNode(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("symbols".into(),v);}
for mut v in data.remove("table_data").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TableData(v)?;
}
wire.insert("table_data".into(),v);}
for mut v in data.remove("ternary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TernaryClass(v)?;
}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("text".into(),v);}
for mut v in data.remove("three_per_em_space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("three_per_em_space".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("unary".into(),v);}
for mut v in data.remove("unary_functions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnaryFunctions(v)?;
}
wire.insert("unary_functions".into(),v);}
for mut v in data.remove("underover").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("underover".into(),v);}
for mut v in data.remove("unicode_symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("unicode_symbols".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_Expression_json(input:&str)->ContractResult<Latexmath_Model_Expression>{let data=decode_Latexmath_Model_Expression(parse_json(input,"Latexmath::Model::Expression@default")?)?; let model:Latexmath_Model_Expression=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_Expression_json(model:&Latexmath_Model_Expression)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_Expression(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_Expression(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::Expression@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("absent") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "absent", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("absent".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "binary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("dividend") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "dividend", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("dividend".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("divisor") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "divisor", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("divisor".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("environment") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "environment", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("environment".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expression") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "expression", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expression".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("items") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "items", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("items".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left_right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "left_right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left_right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("limits") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "limits", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("limits".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("numeric_values") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "numeric_values", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("numeric_values".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("operant") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "operant", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("operant".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("over") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "over", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("over".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "power", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("row_break") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "row_break", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("row_break".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("second_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "second_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("second_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sequence") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "sequence", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sequence".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("subscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "subscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("subscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("substack") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "substack", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("substack".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("supscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "supscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("supscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbols".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table_data") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "table_data", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table_data".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "ternary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("three_per_em_space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "three_per_em_space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("three_per_em_space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_functions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "unary_functions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_functions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("under_over") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "under_over", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("under_over".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("underover") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "underover", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("underover".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unicode_symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "unicode_symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unicode_symbols".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_Expression(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_Expression(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("absent"){
data.insert("absent".into(),v);}
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Latexmath_Model_Base(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("binary"){
if !v.is_null() {
v=decode_Latexmath_Model_Binary(v)?;
}
data.insert("binary".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("dividend"){
if !v.is_null() {
v=decode_Latexmath_Model_Dividend(v)?;
}
data.insert("dividend".into(),v);}
if let Some(mut v)=data.remove("divisor"){
if !v.is_null() {
v=decode_Latexmath_Model_Divisor(v)?;
}
data.insert("divisor".into(),v);}
if let Some(mut v)=data.remove("environment"){
if !v.is_null() {
v=decode_Latexmath_Model_Environment(v)?;
}
data.insert("environment".into(),v);}
if let Some(mut v)=data.remove("expression"){
if !v.is_null() {
v=decode_Latexmath_Model_Expression(v)?;
}
data.insert("expression".into(),v);}
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Latexmath_Model_FirstValue(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("fonts"){
data.insert("fonts".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Latexmath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("items"){
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "items", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=decode_Latexmath_Model_ItemsItem(v)?;
*slot=v; }
}
data.insert("items".into(),v);}
if let Some(mut v)=data.remove("left_right"){
if !v.is_null() {
v=decode_Latexmath_Model_LeftRight(v)?;
}
data.insert("left_right".into(),v);}
if let Some(mut v)=data.remove("limits"){
if !v.is_null() {
v=decode_Latexmath_Model_Limits(v)?;
}
data.insert("limits".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("numeric_values"){
data.insert("numeric_values".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("operant"){
data.insert("operant".into(),v);}
if let Some(mut v)=data.remove("over"){
if !v.is_null() {
v=decode_Latexmath_Model_Over(v)?;
}
data.insert("over".into(),v);}
if let Some(mut v)=data.remove("power"){
if !v.is_null() {
v=decode_Latexmath_Model_Power(v)?;
}
data.insert("power".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Latexmath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("row_break"){
data.insert("row_break".into(),v);}
if let Some(mut v)=data.remove("second_value"){
if !v.is_null() {
v=decode_Latexmath_Model_SecondValue(v)?;
}
data.insert("second_value".into(),v);}
if let Some(mut v)=data.remove("sequence"){
if !v.is_null() {
v=decode_Latexmath_Model_Sequence(v)?;
}
data.insert("sequence".into(),v);}
if let Some(mut v)=data.remove("space"){
data.insert("space".into(),v);}
if let Some(mut v)=data.remove("subscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Subscript(v)?;
}
data.insert("subscript".into(),v);}
if let Some(mut v)=data.remove("substack"){
data.insert("substack".into(),v);}
if let Some(mut v)=data.remove("supscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Supscript(v)?;
}
data.insert("supscript".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Latexmath_Model_SymbolNode(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("symbols".into(),v);}
if let Some(mut v)=data.remove("table_data"){
if !v.is_null() {
v=decode_Latexmath_Model_TableData(v)?;
}
data.insert("table_data".into(),v);}
if let Some(mut v)=data.remove("ternary"){
data.insert("ternary".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
if !v.is_null() {
v=decode_Latexmath_Model_TernaryClass(v)?;
}
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("three_per_em_space"){
data.insert("three_per_em_space".into(),v);}
if let Some(mut v)=data.remove("unary"){
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("unary_functions"){
if !v.is_null() {
v=decode_Latexmath_Model_UnaryFunctions(v)?;
}
data.insert("unary_functions".into(),v);}
if let Some(mut v)=data.remove("under_over"){
if !v.is_null() {
v=decode_Latexmath_Model_UnderOver(v)?;
}
data.insert("under_over".into(),v);}
if let Some(mut v)=data.remove("underover"){
data.insert("underover".into(),v);}
if let Some(mut v)=data.remove("unicode_symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("unicode_symbols".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_Expression(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::Expression@default")?;let mut wire=Map::new();
for mut v in data.remove("absent").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("absent".into(),v);}
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Base(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("binary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Binary(v)?;
}
wire.insert("binary".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("dividend").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Dividend(v)?;
}
wire.insert("dividend".into(),v);}
for mut v in data.remove("divisor").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Divisor(v)?;
}
wire.insert("divisor".into(),v);}
for mut v in data.remove("environment").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Environment(v)?;
}
wire.insert("environment".into(),v);}
for mut v in data.remove("expression").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Expression(v)?;
}
wire.insert("expression".into(),v);}
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_FirstValue(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("items").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
let array=v.as_array_mut().ok_or_else(||failure(ErrorReason::InvalidDocument,"Latexmath::Model::Expression@default", "items", "/input", "collection requires array"))?; for slot in array { let mut v=slot.take();
v=encode_Latexmath_Model_ItemsItem(v)?;
*slot=v; }
}
wire.insert("items".into(),v);}
for mut v in data.remove("left_right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_LeftRight(v)?;
}
wire.insert("left_right".into(),v);}
for mut v in data.remove("limits").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Limits(v)?;
}
wire.insert("limits".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("numeric_values").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("numeric_values".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("operant").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("operant".into(),v);}
for mut v in data.remove("over").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Over(v)?;
}
wire.insert("over".into(),v);}
for mut v in data.remove("power").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Power(v)?;
}
wire.insert("power".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("row_break").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("row_break".into(),v);}
for mut v in data.remove("second_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SecondValue(v)?;
}
wire.insert("second_value".into(),v);}
for mut v in data.remove("sequence").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Sequence(v)?;
}
wire.insert("sequence".into(),v);}
for mut v in data.remove("space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("space".into(),v);}
for mut v in data.remove("subscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Subscript(v)?;
}
wire.insert("subscript".into(),v);}
for mut v in data.remove("substack").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("substack".into(),v);}
for mut v in data.remove("supscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Supscript(v)?;
}
wire.insert("supscript".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SymbolNode(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("symbols".into(),v);}
for mut v in data.remove("table_data").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TableData(v)?;
}
wire.insert("table_data".into(),v);}
for mut v in data.remove("ternary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TernaryClass(v)?;
}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("text".into(),v);}
for mut v in data.remove("three_per_em_space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("three_per_em_space".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("unary".into(),v);}
for mut v in data.remove("unary_functions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnaryFunctions(v)?;
}
wire.insert("unary_functions".into(),v);}
for mut v in data.remove("under_over").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnderOver(v)?;
}
wire.insert("under_over".into(),v);}
for mut v in data.remove("underover").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("underover".into(),v);}
for mut v in data.remove("unicode_symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("unicode_symbols".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_FirstValue_json(input:&str)->ContractResult<Latexmath_Model_FirstValue>{let data=decode_Latexmath_Model_FirstValue(parse_json(input,"Latexmath::Model::FirstValue@default")?)?; let model:Latexmath_Model_FirstValue=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_FirstValue_json(model:&Latexmath_Model_FirstValue)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_FirstValue(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_FirstValue(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::FirstValue@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("absent") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "absent", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("absent".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "binary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("dividend") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "dividend", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("dividend".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("divisor") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "divisor", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("divisor".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("environment") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "environment", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("environment".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expression") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "expression", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expression".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "left", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left_paren") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "left_paren", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left_paren".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("lparen") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "lparen", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("lparen".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("numeric_values") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "numeric_values", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("numeric_values".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("operant") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "operant", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("operant".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("right_paren") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "right_paren", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("right_paren".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("row_break") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "row_break", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("row_break".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("rparen") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "rparen", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("rparen".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("subscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "subscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("subscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("supscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "supscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("supscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbols".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "ternary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text_list") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "text_list", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text_list".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("three_per_em_space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "three_per_em_space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("three_per_em_space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_functions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "unary_functions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_functions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("underover") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "underover", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("underover".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unicode_symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "unicode_symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unicode_symbols".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::FirstValue@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_FirstValue(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_FirstValue(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("absent"){
data.insert("absent".into(),v);}
if let Some(mut v)=data.remove("binary"){
if !v.is_null() {
v=decode_Latexmath_Model_Binary(v)?;
}
data.insert("binary".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("dividend"){
if !v.is_null() {
v=decode_Latexmath_Model_Dividend(v)?;
}
data.insert("dividend".into(),v);}
if let Some(mut v)=data.remove("divisor"){
if !v.is_null() {
v=decode_Latexmath_Model_Divisor(v)?;
}
data.insert("divisor".into(),v);}
if let Some(mut v)=data.remove("environment"){
if !v.is_null() {
v=decode_Latexmath_Model_Environment(v)?;
}
data.insert("environment".into(),v);}
if let Some(mut v)=data.remove("expression"){
if !v.is_null() {
v=decode_Latexmath_Model_Expression(v)?;
}
data.insert("expression".into(),v);}
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Latexmath_Model_FirstValue(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("fonts"){
data.insert("fonts".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Latexmath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("left"){
data.insert("left".into(),v);}
if let Some(mut v)=data.remove("left_paren"){
if !v.is_null() {
v=decode_Latexmath_Model_LeftParen(v)?;
}
data.insert("left_paren".into(),v);}
if let Some(mut v)=data.remove("lparen"){
data.insert("lparen".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("numeric_values"){
data.insert("numeric_values".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("operant"){
data.insert("operant".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Latexmath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("right"){
data.insert("right".into(),v);}
if let Some(mut v)=data.remove("right_paren"){
if !v.is_null() {
v=decode_Latexmath_Model_RightParen(v)?;
}
data.insert("right_paren".into(),v);}
if let Some(mut v)=data.remove("row_break"){
data.insert("row_break".into(),v);}
if let Some(mut v)=data.remove("rparen"){
data.insert("rparen".into(),v);}
if let Some(mut v)=data.remove("space"){
data.insert("space".into(),v);}
if let Some(mut v)=data.remove("subscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Subscript(v)?;
}
data.insert("subscript".into(),v);}
if let Some(mut v)=data.remove("supscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Supscript(v)?;
}
data.insert("supscript".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Latexmath_Model_SymbolNode(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("symbols".into(),v);}
if let Some(mut v)=data.remove("ternary"){
data.insert("ternary".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
if !v.is_null() {
v=decode_Latexmath_Model_TernaryClass(v)?;
}
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("text_list"){
data.insert("text_list".into(),v);}
if let Some(mut v)=data.remove("three_per_em_space"){
data.insert("three_per_em_space".into(),v);}
if let Some(mut v)=data.remove("unary"){
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("unary_functions"){
if !v.is_null() {
v=decode_Latexmath_Model_UnaryFunctions(v)?;
}
data.insert("unary_functions".into(),v);}
if let Some(mut v)=data.remove("underover"){
data.insert("underover".into(),v);}
if let Some(mut v)=data.remove("unicode_symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("unicode_symbols".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_FirstValue(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::FirstValue@default")?;let mut wire=Map::new();
for mut v in data.remove("absent").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("absent".into(),v);}
for mut v in data.remove("binary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Binary(v)?;
}
wire.insert("binary".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("dividend").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Dividend(v)?;
}
wire.insert("dividend".into(),v);}
for mut v in data.remove("divisor").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Divisor(v)?;
}
wire.insert("divisor".into(),v);}
for mut v in data.remove("environment").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Environment(v)?;
}
wire.insert("environment".into(),v);}
for mut v in data.remove("expression").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Expression(v)?;
}
wire.insert("expression".into(),v);}
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_FirstValue(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("left").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("left".into(),v);}
for mut v in data.remove("left_paren").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_LeftParen(v)?;
}
wire.insert("left_paren".into(),v);}
for mut v in data.remove("lparen").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("lparen".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("numeric_values").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("numeric_values".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("operant").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("operant".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("right".into(),v);}
for mut v in data.remove("right_paren").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_RightParen(v)?;
}
wire.insert("right_paren".into(),v);}
for mut v in data.remove("row_break").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("row_break".into(),v);}
for mut v in data.remove("rparen").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("rparen".into(),v);}
for mut v in data.remove("space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("space".into(),v);}
for mut v in data.remove("subscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Subscript(v)?;
}
wire.insert("subscript".into(),v);}
for mut v in data.remove("supscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Supscript(v)?;
}
wire.insert("supscript".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SymbolNode(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("symbols".into(),v);}
for mut v in data.remove("ternary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TernaryClass(v)?;
}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("text".into(),v);}
for mut v in data.remove("text_list").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("text_list".into(),v);}
for mut v in data.remove("three_per_em_space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("three_per_em_space".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("unary".into(),v);}
for mut v in data.remove("unary_functions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnaryFunctions(v)?;
}
wire.insert("unary_functions".into(),v);}
for mut v in data.remove("underover").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("underover".into(),v);}
for mut v in data.remove("unicode_symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("unicode_symbols".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_IntermediateExp_json(input:&str)->ContractResult<Latexmath_Model_IntermediateExp>{let data=decode_Latexmath_Model_IntermediateExp(parse_json(input,"Latexmath::Model::IntermediateExp@default")?)?; let model:Latexmath_Model_IntermediateExp=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_IntermediateExp_json(model:&Latexmath_Model_IntermediateExp)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_IntermediateExp(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_IntermediateExp(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::IntermediateExp@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("binary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "binary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("environment") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "environment", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("environment".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expression") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "expression", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expression".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left_paren") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "left_paren", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left_paren".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("numeric_values") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "numeric_values", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("numeric_values".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("operant") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "operant", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("operant".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("right_paren") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "right_paren", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("right_paren".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("row_break") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "row_break", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("row_break".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("second_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "second_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("second_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("subscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "subscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("subscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("supscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "supscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("supscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbols".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "ternary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("three_per_em_space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "three_per_em_space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("three_per_em_space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_functions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "unary_functions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_functions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("under_over") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "under_over", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("under_over".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("underover") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "underover", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("underover".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unicode_symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "unicode_symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unicode_symbols".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::IntermediateExp@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_IntermediateExp(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_IntermediateExp(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("binary"){
if !v.is_null() {
v=decode_Latexmath_Model_Binary(v)?;
}
data.insert("binary".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("environment"){
if !v.is_null() {
v=decode_Latexmath_Model_Environment(v)?;
}
data.insert("environment".into(),v);}
if let Some(mut v)=data.remove("expression"){
if !v.is_null() {
v=decode_Latexmath_Model_Expression(v)?;
}
data.insert("expression".into(),v);}
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Latexmath_Model_FirstValue(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("fonts"){
data.insert("fonts".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Latexmath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("left_paren"){
if !v.is_null() {
v=decode_Latexmath_Model_LeftParen(v)?;
}
data.insert("left_paren".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("numeric_values"){
data.insert("numeric_values".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("operant"){
data.insert("operant".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Latexmath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("right_paren"){
if !v.is_null() {
v=decode_Latexmath_Model_RightParen(v)?;
}
data.insert("right_paren".into(),v);}
if let Some(mut v)=data.remove("row_break"){
data.insert("row_break".into(),v);}
if let Some(mut v)=data.remove("second_value"){
if !v.is_null() {
v=decode_Latexmath_Model_SecondValue(v)?;
}
data.insert("second_value".into(),v);}
if let Some(mut v)=data.remove("space"){
data.insert("space".into(),v);}
if let Some(mut v)=data.remove("subscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Subscript(v)?;
}
data.insert("subscript".into(),v);}
if let Some(mut v)=data.remove("supscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Supscript(v)?;
}
data.insert("supscript".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Latexmath_Model_SymbolNode(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("symbols".into(),v);}
if let Some(mut v)=data.remove("ternary"){
data.insert("ternary".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
if !v.is_null() {
v=decode_Latexmath_Model_TernaryClass(v)?;
}
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("three_per_em_space"){
data.insert("three_per_em_space".into(),v);}
if let Some(mut v)=data.remove("unary"){
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("unary_functions"){
if !v.is_null() {
v=decode_Latexmath_Model_UnaryFunctions(v)?;
}
data.insert("unary_functions".into(),v);}
if let Some(mut v)=data.remove("under_over"){
if !v.is_null() {
v=decode_Latexmath_Model_UnderOver(v)?;
}
data.insert("under_over".into(),v);}
if let Some(mut v)=data.remove("underover"){
data.insert("underover".into(),v);}
if let Some(mut v)=data.remove("unicode_symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("unicode_symbols".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_IntermediateExp(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::IntermediateExp@default")?;let mut wire=Map::new();
for mut v in data.remove("binary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Binary(v)?;
}
wire.insert("binary".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("environment").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Environment(v)?;
}
wire.insert("environment".into(),v);}
for mut v in data.remove("expression").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Expression(v)?;
}
wire.insert("expression".into(),v);}
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_FirstValue(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("left_paren").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_LeftParen(v)?;
}
wire.insert("left_paren".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("numeric_values").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("numeric_values".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("operant").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("operant".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("right_paren").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_RightParen(v)?;
}
wire.insert("right_paren".into(),v);}
for mut v in data.remove("row_break").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("row_break".into(),v);}
for mut v in data.remove("second_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SecondValue(v)?;
}
wire.insert("second_value".into(),v);}
for mut v in data.remove("space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("space".into(),v);}
for mut v in data.remove("subscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Subscript(v)?;
}
wire.insert("subscript".into(),v);}
for mut v in data.remove("supscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Supscript(v)?;
}
wire.insert("supscript".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SymbolNode(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("symbols".into(),v);}
for mut v in data.remove("ternary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TernaryClass(v)?;
}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("text".into(),v);}
for mut v in data.remove("three_per_em_space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("three_per_em_space".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("unary".into(),v);}
for mut v in data.remove("unary_functions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnaryFunctions(v)?;
}
wire.insert("unary_functions".into(),v);}
for mut v in data.remove("under_over").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnderOver(v)?;
}
wire.insert("under_over".into(),v);}
for mut v in data.remove("underover").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("underover".into(),v);}
for mut v in data.remove("unicode_symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("unicode_symbols".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_ItemsItem_json(input:&str)->ContractResult<Latexmath_Model_ItemsItem>{let data=decode_Latexmath_Model_ItemsItem(parse_json(input,"Latexmath::Model::ItemsItem@default")?)?; let model:Latexmath_Model_ItemsItem=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_ItemsItem_json(model:&Latexmath_Model_ItemsItem)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_ItemsItem(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_ItemsItem(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::ItemsItem@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "binary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("dividend") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "dividend", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("dividend".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("divisor") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "divisor", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("divisor".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("environment") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "environment", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("environment".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expression") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "expression", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expression".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left_right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "left_right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left_right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("limits") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "limits", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("limits".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("numeric_values") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "numeric_values", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("numeric_values".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("operant") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "operant", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("operant".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("over") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "over", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("over".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "power", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("row_break") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "row_break", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("row_break".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("second_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "second_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("second_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sequence") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "sequence", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sequence".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("subscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "subscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("subscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("substack") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "substack", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("substack".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("supscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "supscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("supscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbols".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table_data") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "table_data", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table_data".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "ternary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("three_per_em_space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "three_per_em_space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("three_per_em_space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_functions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "unary_functions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_functions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("under_over") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "under_over", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("under_over".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("underover") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "underover", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("underover".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unicode_symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "unicode_symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unicode_symbols".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ItemsItem@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_ItemsItem(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_ItemsItem(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Latexmath_Model_Base(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("binary"){
if !v.is_null() {
v=decode_Latexmath_Model_Binary(v)?;
}
data.insert("binary".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("dividend"){
if !v.is_null() {
v=decode_Latexmath_Model_Dividend(v)?;
}
data.insert("dividend".into(),v);}
if let Some(mut v)=data.remove("divisor"){
if !v.is_null() {
v=decode_Latexmath_Model_Divisor(v)?;
}
data.insert("divisor".into(),v);}
if let Some(mut v)=data.remove("environment"){
if !v.is_null() {
v=decode_Latexmath_Model_Environment(v)?;
}
data.insert("environment".into(),v);}
if let Some(mut v)=data.remove("expression"){
if !v.is_null() {
v=decode_Latexmath_Model_Expression(v)?;
}
data.insert("expression".into(),v);}
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Latexmath_Model_FirstValue(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("fonts"){
data.insert("fonts".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Latexmath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("left_right"){
if !v.is_null() {
v=decode_Latexmath_Model_LeftRight(v)?;
}
data.insert("left_right".into(),v);}
if let Some(mut v)=data.remove("limits"){
if !v.is_null() {
v=decode_Latexmath_Model_Limits(v)?;
}
data.insert("limits".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("numeric_values"){
data.insert("numeric_values".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("operant"){
data.insert("operant".into(),v);}
if let Some(mut v)=data.remove("over"){
if !v.is_null() {
v=decode_Latexmath_Model_Over(v)?;
}
data.insert("over".into(),v);}
if let Some(mut v)=data.remove("power"){
if !v.is_null() {
v=decode_Latexmath_Model_Power(v)?;
}
data.insert("power".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Latexmath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("row_break"){
data.insert("row_break".into(),v);}
if let Some(mut v)=data.remove("second_value"){
if !v.is_null() {
v=decode_Latexmath_Model_SecondValue(v)?;
}
data.insert("second_value".into(),v);}
if let Some(mut v)=data.remove("sequence"){
if !v.is_null() {
v=decode_Latexmath_Model_Sequence(v)?;
}
data.insert("sequence".into(),v);}
if let Some(mut v)=data.remove("space"){
data.insert("space".into(),v);}
if let Some(mut v)=data.remove("subscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Subscript(v)?;
}
data.insert("subscript".into(),v);}
if let Some(mut v)=data.remove("substack"){
data.insert("substack".into(),v);}
if let Some(mut v)=data.remove("supscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Supscript(v)?;
}
data.insert("supscript".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Latexmath_Model_SymbolNode(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("symbols".into(),v);}
if let Some(mut v)=data.remove("table_data"){
if !v.is_null() {
v=decode_Latexmath_Model_TableData(v)?;
}
data.insert("table_data".into(),v);}
if let Some(mut v)=data.remove("ternary"){
data.insert("ternary".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
if !v.is_null() {
v=decode_Latexmath_Model_TernaryClass(v)?;
}
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("three_per_em_space"){
data.insert("three_per_em_space".into(),v);}
if let Some(mut v)=data.remove("unary"){
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("unary_functions"){
if !v.is_null() {
v=decode_Latexmath_Model_UnaryFunctions(v)?;
}
data.insert("unary_functions".into(),v);}
if let Some(mut v)=data.remove("under_over"){
if !v.is_null() {
v=decode_Latexmath_Model_UnderOver(v)?;
}
data.insert("under_over".into(),v);}
if let Some(mut v)=data.remove("underover"){
data.insert("underover".into(),v);}
if let Some(mut v)=data.remove("unicode_symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("unicode_symbols".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_ItemsItem(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::ItemsItem@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Base(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("binary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Binary(v)?;
}
wire.insert("binary".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("dividend").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Dividend(v)?;
}
wire.insert("dividend".into(),v);}
for mut v in data.remove("divisor").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Divisor(v)?;
}
wire.insert("divisor".into(),v);}
for mut v in data.remove("environment").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Environment(v)?;
}
wire.insert("environment".into(),v);}
for mut v in data.remove("expression").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Expression(v)?;
}
wire.insert("expression".into(),v);}
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_FirstValue(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("left_right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_LeftRight(v)?;
}
wire.insert("left_right".into(),v);}
for mut v in data.remove("limits").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Limits(v)?;
}
wire.insert("limits".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("numeric_values").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("numeric_values".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("operant").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("operant".into(),v);}
for mut v in data.remove("over").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Over(v)?;
}
wire.insert("over".into(),v);}
for mut v in data.remove("power").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Power(v)?;
}
wire.insert("power".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("row_break").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("row_break".into(),v);}
for mut v in data.remove("second_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SecondValue(v)?;
}
wire.insert("second_value".into(),v);}
for mut v in data.remove("sequence").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Sequence(v)?;
}
wire.insert("sequence".into(),v);}
for mut v in data.remove("space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("space".into(),v);}
for mut v in data.remove("subscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Subscript(v)?;
}
wire.insert("subscript".into(),v);}
for mut v in data.remove("substack").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("substack".into(),v);}
for mut v in data.remove("supscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Supscript(v)?;
}
wire.insert("supscript".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SymbolNode(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("symbols".into(),v);}
for mut v in data.remove("table_data").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TableData(v)?;
}
wire.insert("table_data".into(),v);}
for mut v in data.remove("ternary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TernaryClass(v)?;
}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("text".into(),v);}
for mut v in data.remove("three_per_em_space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("three_per_em_space".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("unary".into(),v);}
for mut v in data.remove("unary_functions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnaryFunctions(v)?;
}
wire.insert("unary_functions".into(),v);}
for mut v in data.remove("under_over").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnderOver(v)?;
}
wire.insert("under_over".into(),v);}
for mut v in data.remove("underover").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("underover".into(),v);}
for mut v in data.remove("unicode_symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("unicode_symbols".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_LatexTree_json(input:&str)->ContractResult<Latexmath_Model_LatexTree>{let data=decode_Latexmath_Model_LatexTree(parse_json(input,"Latexmath::Model::LatexTree@default")?)?; let model:Latexmath_Model_LatexTree=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_LatexTree_json(model:&Latexmath_Model_LatexTree)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_LatexTree(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_LatexTree(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::LatexTree@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "binary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("dividend") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "dividend", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("dividend".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("divisor") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "divisor", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("divisor".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("environment") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "environment", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("environment".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expression") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "expression", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expression".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left_right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "left_right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left_right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("limits") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "limits", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("limits".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("numeric_values") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "numeric_values", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("numeric_values".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("operant") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "operant", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("operant".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("over") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "over", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("over".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "power", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("row_break") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "row_break", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("row_break".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("second_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "second_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("second_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sequence") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "sequence", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sequence".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("subscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "subscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("subscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("substack") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "substack", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("substack".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("supscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "supscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("supscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbols".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table_data") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "table_data", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table_data".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "ternary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("three_per_em_space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "three_per_em_space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("three_per_em_space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_functions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "unary_functions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_functions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("under_over") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "under_over", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("under_over".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("underover") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "underover", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("underover".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unicode_symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "unicode_symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unicode_symbols".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LatexTree@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_LatexTree(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_LatexTree(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Latexmath_Model_Base(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("binary"){
if !v.is_null() {
v=decode_Latexmath_Model_Binary(v)?;
}
data.insert("binary".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("dividend"){
if !v.is_null() {
v=decode_Latexmath_Model_Dividend(v)?;
}
data.insert("dividend".into(),v);}
if let Some(mut v)=data.remove("divisor"){
if !v.is_null() {
v=decode_Latexmath_Model_Divisor(v)?;
}
data.insert("divisor".into(),v);}
if let Some(mut v)=data.remove("environment"){
if !v.is_null() {
v=decode_Latexmath_Model_Environment(v)?;
}
data.insert("environment".into(),v);}
if let Some(mut v)=data.remove("expression"){
if !v.is_null() {
v=decode_Latexmath_Model_Expression(v)?;
}
data.insert("expression".into(),v);}
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Latexmath_Model_FirstValue(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("fonts"){
data.insert("fonts".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Latexmath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("left_right"){
if !v.is_null() {
v=decode_Latexmath_Model_LeftRight(v)?;
}
data.insert("left_right".into(),v);}
if let Some(mut v)=data.remove("limits"){
if !v.is_null() {
v=decode_Latexmath_Model_Limits(v)?;
}
data.insert("limits".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("numeric_values"){
data.insert("numeric_values".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("operant"){
data.insert("operant".into(),v);}
if let Some(mut v)=data.remove("over"){
if !v.is_null() {
v=decode_Latexmath_Model_Over(v)?;
}
data.insert("over".into(),v);}
if let Some(mut v)=data.remove("power"){
if !v.is_null() {
v=decode_Latexmath_Model_Power(v)?;
}
data.insert("power".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Latexmath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("row_break"){
data.insert("row_break".into(),v);}
if let Some(mut v)=data.remove("second_value"){
if !v.is_null() {
v=decode_Latexmath_Model_SecondValue(v)?;
}
data.insert("second_value".into(),v);}
if let Some(mut v)=data.remove("sequence"){
if !v.is_null() {
v=decode_Latexmath_Model_Sequence(v)?;
}
data.insert("sequence".into(),v);}
if let Some(mut v)=data.remove("space"){
data.insert("space".into(),v);}
if let Some(mut v)=data.remove("subscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Subscript(v)?;
}
data.insert("subscript".into(),v);}
if let Some(mut v)=data.remove("substack"){
data.insert("substack".into(),v);}
if let Some(mut v)=data.remove("supscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Supscript(v)?;
}
data.insert("supscript".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Latexmath_Model_SymbolNode(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("symbols".into(),v);}
if let Some(mut v)=data.remove("table_data"){
if !v.is_null() {
v=decode_Latexmath_Model_TableData(v)?;
}
data.insert("table_data".into(),v);}
if let Some(mut v)=data.remove("ternary"){
data.insert("ternary".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
if !v.is_null() {
v=decode_Latexmath_Model_TernaryClass(v)?;
}
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("three_per_em_space"){
data.insert("three_per_em_space".into(),v);}
if let Some(mut v)=data.remove("unary"){
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("unary_functions"){
if !v.is_null() {
v=decode_Latexmath_Model_UnaryFunctions(v)?;
}
data.insert("unary_functions".into(),v);}
if let Some(mut v)=data.remove("under_over"){
if !v.is_null() {
v=decode_Latexmath_Model_UnderOver(v)?;
}
data.insert("under_over".into(),v);}
if let Some(mut v)=data.remove("underover"){
data.insert("underover".into(),v);}
if let Some(mut v)=data.remove("unicode_symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("unicode_symbols".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_LatexTree(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::LatexTree@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Base(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("binary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Binary(v)?;
}
wire.insert("binary".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("dividend").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Dividend(v)?;
}
wire.insert("dividend".into(),v);}
for mut v in data.remove("divisor").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Divisor(v)?;
}
wire.insert("divisor".into(),v);}
for mut v in data.remove("environment").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Environment(v)?;
}
wire.insert("environment".into(),v);}
for mut v in data.remove("expression").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Expression(v)?;
}
wire.insert("expression".into(),v);}
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_FirstValue(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("left_right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_LeftRight(v)?;
}
wire.insert("left_right".into(),v);}
for mut v in data.remove("limits").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Limits(v)?;
}
wire.insert("limits".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("numeric_values").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("numeric_values".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("operant").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("operant".into(),v);}
for mut v in data.remove("over").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Over(v)?;
}
wire.insert("over".into(),v);}
for mut v in data.remove("power").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Power(v)?;
}
wire.insert("power".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("row_break").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("row_break".into(),v);}
for mut v in data.remove("second_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SecondValue(v)?;
}
wire.insert("second_value".into(),v);}
for mut v in data.remove("sequence").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Sequence(v)?;
}
wire.insert("sequence".into(),v);}
for mut v in data.remove("space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("space".into(),v);}
for mut v in data.remove("subscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Subscript(v)?;
}
wire.insert("subscript".into(),v);}
for mut v in data.remove("substack").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("substack".into(),v);}
for mut v in data.remove("supscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Supscript(v)?;
}
wire.insert("supscript".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SymbolNode(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("symbols".into(),v);}
for mut v in data.remove("table_data").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TableData(v)?;
}
wire.insert("table_data".into(),v);}
for mut v in data.remove("ternary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TernaryClass(v)?;
}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("text".into(),v);}
for mut v in data.remove("three_per_em_space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("three_per_em_space".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("unary".into(),v);}
for mut v in data.remove("unary_functions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnaryFunctions(v)?;
}
wire.insert("unary_functions".into(),v);}
for mut v in data.remove("under_over").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnderOver(v)?;
}
wire.insert("under_over".into(),v);}
for mut v in data.remove("underover").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("underover".into(),v);}
for mut v in data.remove("unicode_symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("unicode_symbols".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_LeftParen_json(input:&str)->ContractResult<Latexmath_Model_LeftParen>{let data=decode_Latexmath_Model_LeftParen(parse_json(input,"Latexmath::Model::LeftParen@default")?)?; let model:Latexmath_Model_LeftParen=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::LeftParen@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_LeftParen_json(model:&Latexmath_Model_LeftParen)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::LeftParen@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_LeftParen(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::LeftParen@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_LeftParen(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::LeftParen@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("literal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LeftParen@default", "literal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("lparen") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LeftParen@default", "lparen", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("lparen".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LeftParen@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_LeftParen(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_LeftParen(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("literal"){
data.insert("literal".into(),v);}
if let Some(mut v)=data.remove("lparen"){
data.insert("lparen".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_LeftParen(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::LeftParen@default")?;let mut wire=Map::new();
for mut v in data.remove("literal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal".into(),v);}
for mut v in data.remove("lparen").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("lparen".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_LeftRight_json(input:&str)->ContractResult<Latexmath_Model_LeftRight>{let data=decode_Latexmath_Model_LeftRight(parse_json(input,"Latexmath::Model::LeftRight@default")?)?; let model:Latexmath_Model_LeftRight=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::LeftRight@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_LeftRight_json(model:&Latexmath_Model_LeftRight)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::LeftRight@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_LeftRight(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::LeftRight@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_LeftRight(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::LeftRight@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("dividend") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LeftRight@default", "dividend", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("dividend".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("divisor") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LeftRight@default", "divisor", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("divisor".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expression") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LeftRight@default", "expression", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expression".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LeftRight@default", "left", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left_paren") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LeftRight@default", "left_paren", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left_paren".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LeftRight@default", "right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("right_paren") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LeftRight@default", "right_paren", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("right_paren".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::LeftRight@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_LeftRight(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_LeftRight(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("dividend"){
if !v.is_null() {
v=decode_Latexmath_Model_Dividend(v)?;
}
data.insert("dividend".into(),v);}
if let Some(mut v)=data.remove("divisor"){
if !v.is_null() {
v=decode_Latexmath_Model_Divisor(v)?;
}
data.insert("divisor".into(),v);}
if let Some(mut v)=data.remove("expression"){
if !v.is_null() {
v=decode_Latexmath_Model_Expression(v)?;
}
data.insert("expression".into(),v);}
if let Some(mut v)=data.remove("left"){
data.insert("left".into(),v);}
if let Some(mut v)=data.remove("left_paren"){
if !v.is_null() {
v=decode_Latexmath_Model_LeftParen(v)?;
}
data.insert("left_paren".into(),v);}
if let Some(mut v)=data.remove("right"){
data.insert("right".into(),v);}
if let Some(mut v)=data.remove("right_paren"){
if !v.is_null() {
v=decode_Latexmath_Model_RightParen(v)?;
}
data.insert("right_paren".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_LeftRight(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::LeftRight@default")?;let mut wire=Map::new();
for mut v in data.remove("dividend").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Dividend(v)?;
}
wire.insert("dividend".into(),v);}
for mut v in data.remove("divisor").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Divisor(v)?;
}
wire.insert("divisor".into(),v);}
for mut v in data.remove("expression").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Expression(v)?;
}
wire.insert("expression".into(),v);}
for mut v in data.remove("left").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("left".into(),v);}
for mut v in data.remove("left_paren").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_LeftParen(v)?;
}
wire.insert("left_paren".into(),v);}
for mut v in data.remove("right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("right".into(),v);}
for mut v in data.remove("right_paren").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_RightParen(v)?;
}
wire.insert("right_paren".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_Limits_json(input:&str)->ContractResult<Latexmath_Model_Limits>{let data=decode_Latexmath_Model_Limits(parse_json(input,"Latexmath::Model::Limits@default")?)?; let model:Latexmath_Model_Limits=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Limits@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_Limits_json(model:&Latexmath_Model_Limits)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Limits@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_Limits(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Limits@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_Limits(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::Limits@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Limits@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Limits@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Limits@default", "power", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Limits@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_Limits(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_Limits(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Latexmath_Model_Base(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Latexmath_Model_FirstValue(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("power"){
if !v.is_null() {
v=decode_Latexmath_Model_Power(v)?;
}
data.insert("power".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_Limits(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::Limits@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Base(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_FirstValue(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("power").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Power(v)?;
}
wire.insert("power".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_MathSymbol_json(input:&str)->ContractResult<Latexmath_Model_MathSymbol>{let data=decode_Latexmath_Model_MathSymbol(parse_json(input,"Latexmath::Model::MathSymbol@default")?)?; let model:Latexmath_Model_MathSymbol=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::MathSymbol@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_MathSymbol_json(model:&Latexmath_Model_MathSymbol)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::MathSymbol@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_MathSymbol(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::MathSymbol@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_MathSymbol(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::MathSymbol@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("spelling") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::MathSymbol@default", "spelling", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("spelling".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::MathSymbol@default", "value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("kind") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::MathSymbol@default", "kind", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("kind".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::MathSymbol@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_MathSymbol(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_MathSymbol(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("spelling"){
data.insert("spelling".into(),v);}
if let Some(mut v)=data.remove("value"){
data.insert("value".into(),v);}
if let Some(mut v)=data.remove("kind"){
data.insert("kind".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_MathSymbol(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::MathSymbol@default")?;let mut wire=Map::new();
for mut v in data.remove("spelling").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("spelling".into(),v);}
for mut v in data.remove("value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("value".into(),v);}
for mut v in data.remove("kind").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("kind".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_Options_json(input:&str)->ContractResult<Latexmath_Model_Options>{let data=decode_Latexmath_Model_Options(parse_json(input,"Latexmath::Model::Options@default")?)?; let model:Latexmath_Model_Options=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_Options_json(model:&Latexmath_Model_Options)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_Options(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_Options(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::Options@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("absent") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "absent", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("absent".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "binary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("environment") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "environment", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("environment".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expression") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "expression", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expression".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("numeric_values") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "numeric_values", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("numeric_values".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("operant") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "operant", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("operant".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("row_break") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "row_break", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("row_break".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("subscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "subscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("subscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("supscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "supscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("supscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbols".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "ternary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("three_per_em_space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "three_per_em_space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("three_per_em_space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_functions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "unary_functions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_functions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("underover") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "underover", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("underover".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unicode_symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "unicode_symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unicode_symbols".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Options@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_Options(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_Options(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("absent"){
data.insert("absent".into(),v);}
if let Some(mut v)=data.remove("binary"){
if !v.is_null() {
v=decode_Latexmath_Model_Binary(v)?;
}
data.insert("binary".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("environment"){
if !v.is_null() {
v=decode_Latexmath_Model_Environment(v)?;
}
data.insert("environment".into(),v);}
if let Some(mut v)=data.remove("expression"){
if !v.is_null() {
v=decode_Latexmath_Model_Expression(v)?;
}
data.insert("expression".into(),v);}
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Latexmath_Model_FirstValue(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("fonts"){
data.insert("fonts".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Latexmath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("numeric_values"){
data.insert("numeric_values".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("operant"){
data.insert("operant".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Latexmath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("row_break"){
data.insert("row_break".into(),v);}
if let Some(mut v)=data.remove("space"){
data.insert("space".into(),v);}
if let Some(mut v)=data.remove("subscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Subscript(v)?;
}
data.insert("subscript".into(),v);}
if let Some(mut v)=data.remove("supscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Supscript(v)?;
}
data.insert("supscript".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Latexmath_Model_SymbolNode(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("symbols".into(),v);}
if let Some(mut v)=data.remove("ternary"){
data.insert("ternary".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
if !v.is_null() {
v=decode_Latexmath_Model_TernaryClass(v)?;
}
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("three_per_em_space"){
data.insert("three_per_em_space".into(),v);}
if let Some(mut v)=data.remove("unary"){
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("unary_functions"){
if !v.is_null() {
v=decode_Latexmath_Model_UnaryFunctions(v)?;
}
data.insert("unary_functions".into(),v);}
if let Some(mut v)=data.remove("underover"){
data.insert("underover".into(),v);}
if let Some(mut v)=data.remove("unicode_symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("unicode_symbols".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_Options(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::Options@default")?;let mut wire=Map::new();
for mut v in data.remove("absent").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("absent".into(),v);}
for mut v in data.remove("binary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Binary(v)?;
}
wire.insert("binary".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("environment").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Environment(v)?;
}
wire.insert("environment".into(),v);}
for mut v in data.remove("expression").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Expression(v)?;
}
wire.insert("expression".into(),v);}
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_FirstValue(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("numeric_values").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("numeric_values".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("operant").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("operant".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("row_break").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("row_break".into(),v);}
for mut v in data.remove("space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("space".into(),v);}
for mut v in data.remove("subscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Subscript(v)?;
}
wire.insert("subscript".into(),v);}
for mut v in data.remove("supscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Supscript(v)?;
}
wire.insert("supscript".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SymbolNode(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("symbols".into(),v);}
for mut v in data.remove("ternary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TernaryClass(v)?;
}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("text".into(),v);}
for mut v in data.remove("three_per_em_space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("three_per_em_space".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("unary".into(),v);}
for mut v in data.remove("unary_functions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnaryFunctions(v)?;
}
wire.insert("unary_functions".into(),v);}
for mut v in data.remove("underover").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("underover".into(),v);}
for mut v in data.remove("unicode_symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("unicode_symbols".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_Over_json(input:&str)->ContractResult<Latexmath_Model_Over>{let data=decode_Latexmath_Model_Over(parse_json(input,"Latexmath::Model::Over@default")?)?; let model:Latexmath_Model_Over=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_Over_json(model:&Latexmath_Model_Over)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_Over(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_Over(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::Over@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "binary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("dividend") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "dividend", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("dividend".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("divisor") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "divisor", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("divisor".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("environment") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "environment", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("environment".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expression") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "expression", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expression".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("numeric_values") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "numeric_values", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("numeric_values".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("operant") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "operant", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("operant".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "power", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("row_break") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "row_break", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("row_break".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("subscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "subscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("subscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("supscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "supscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("supscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbols".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "ternary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("three_per_em_space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "three_per_em_space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("three_per_em_space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_functions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "unary_functions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_functions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("underover") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "underover", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("underover".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unicode_symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "unicode_symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unicode_symbols".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Over@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_Over(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_Over(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Latexmath_Model_Base(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("binary"){
if !v.is_null() {
v=decode_Latexmath_Model_Binary(v)?;
}
data.insert("binary".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("dividend"){
if !v.is_null() {
v=decode_Latexmath_Model_Dividend(v)?;
}
data.insert("dividend".into(),v);}
if let Some(mut v)=data.remove("divisor"){
if !v.is_null() {
v=decode_Latexmath_Model_Divisor(v)?;
}
data.insert("divisor".into(),v);}
if let Some(mut v)=data.remove("environment"){
if !v.is_null() {
v=decode_Latexmath_Model_Environment(v)?;
}
data.insert("environment".into(),v);}
if let Some(mut v)=data.remove("expression"){
if !v.is_null() {
v=decode_Latexmath_Model_Expression(v)?;
}
data.insert("expression".into(),v);}
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Latexmath_Model_FirstValue(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("fonts"){
data.insert("fonts".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Latexmath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("numeric_values"){
data.insert("numeric_values".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("operant"){
data.insert("operant".into(),v);}
if let Some(mut v)=data.remove("power"){
if !v.is_null() {
v=decode_Latexmath_Model_Power(v)?;
}
data.insert("power".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Latexmath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("row_break"){
data.insert("row_break".into(),v);}
if let Some(mut v)=data.remove("space"){
data.insert("space".into(),v);}
if let Some(mut v)=data.remove("subscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Subscript(v)?;
}
data.insert("subscript".into(),v);}
if let Some(mut v)=data.remove("supscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Supscript(v)?;
}
data.insert("supscript".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Latexmath_Model_SymbolNode(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("symbols".into(),v);}
if let Some(mut v)=data.remove("ternary"){
data.insert("ternary".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
if !v.is_null() {
v=decode_Latexmath_Model_TernaryClass(v)?;
}
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("three_per_em_space"){
data.insert("three_per_em_space".into(),v);}
if let Some(mut v)=data.remove("unary"){
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("unary_functions"){
if !v.is_null() {
v=decode_Latexmath_Model_UnaryFunctions(v)?;
}
data.insert("unary_functions".into(),v);}
if let Some(mut v)=data.remove("underover"){
data.insert("underover".into(),v);}
if let Some(mut v)=data.remove("unicode_symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("unicode_symbols".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_Over(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::Over@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Base(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("binary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Binary(v)?;
}
wire.insert("binary".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("dividend").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Dividend(v)?;
}
wire.insert("dividend".into(),v);}
for mut v in data.remove("divisor").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Divisor(v)?;
}
wire.insert("divisor".into(),v);}
for mut v in data.remove("environment").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Environment(v)?;
}
wire.insert("environment".into(),v);}
for mut v in data.remove("expression").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Expression(v)?;
}
wire.insert("expression".into(),v);}
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_FirstValue(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("numeric_values").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("numeric_values".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("operant").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("operant".into(),v);}
for mut v in data.remove("power").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Power(v)?;
}
wire.insert("power".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("row_break").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("row_break".into(),v);}
for mut v in data.remove("space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("space".into(),v);}
for mut v in data.remove("subscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Subscript(v)?;
}
wire.insert("subscript".into(),v);}
for mut v in data.remove("supscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Supscript(v)?;
}
wire.insert("supscript".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SymbolNode(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("symbols".into(),v);}
for mut v in data.remove("ternary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TernaryClass(v)?;
}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("text".into(),v);}
for mut v in data.remove("three_per_em_space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("three_per_em_space".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("unary".into(),v);}
for mut v in data.remove("unary_functions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnaryFunctions(v)?;
}
wire.insert("unary_functions".into(),v);}
for mut v in data.remove("underover").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("underover".into(),v);}
for mut v in data.remove("unicode_symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("unicode_symbols".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_Power_json(input:&str)->ContractResult<Latexmath_Model_Power>{let data=decode_Latexmath_Model_Power(parse_json(input,"Latexmath::Model::Power@default")?)?; let model:Latexmath_Model_Power=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_Power_json(model:&Latexmath_Model_Power)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_Power(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_Power(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::Power@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("binary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "binary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_functions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "binary_functions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_functions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("environment") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "environment", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("environment".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expression") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "expression", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expression".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left_right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "left_right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left_right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("numeric_values") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "numeric_values", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("numeric_values".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("operant") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "operant", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("operant".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("row_break") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "row_break", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("row_break".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("subscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "subscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("subscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("supscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "supscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("supscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbols".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "ternary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("three_per_em_space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "three_per_em_space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("three_per_em_space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_functions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "unary_functions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_functions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("underover") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "underover", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("underover".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unicode_symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "unicode_symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unicode_symbols".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Power@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_Power(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_Power(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("binary"){
if !v.is_null() {
v=decode_Latexmath_Model_Binary(v)?;
}
data.insert("binary".into(),v);}
if let Some(mut v)=data.remove("binary_functions"){
if !v.is_null() {
v=decode_Latexmath_Model_BinaryFunctions(v)?;
}
data.insert("binary_functions".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("environment"){
if !v.is_null() {
v=decode_Latexmath_Model_Environment(v)?;
}
data.insert("environment".into(),v);}
if let Some(mut v)=data.remove("expression"){
if !v.is_null() {
v=decode_Latexmath_Model_Expression(v)?;
}
data.insert("expression".into(),v);}
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Latexmath_Model_FirstValue(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("fonts"){
data.insert("fonts".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Latexmath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("left_right"){
if !v.is_null() {
v=decode_Latexmath_Model_LeftRight(v)?;
}
data.insert("left_right".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("numeric_values"){
data.insert("numeric_values".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("operant"){
data.insert("operant".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Latexmath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("row_break"){
data.insert("row_break".into(),v);}
if let Some(mut v)=data.remove("space"){
data.insert("space".into(),v);}
if let Some(mut v)=data.remove("subscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Subscript(v)?;
}
data.insert("subscript".into(),v);}
if let Some(mut v)=data.remove("supscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Supscript(v)?;
}
data.insert("supscript".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Latexmath_Model_SymbolNode(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("symbols".into(),v);}
if let Some(mut v)=data.remove("ternary"){
data.insert("ternary".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
if !v.is_null() {
v=decode_Latexmath_Model_TernaryClass(v)?;
}
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("three_per_em_space"){
data.insert("three_per_em_space".into(),v);}
if let Some(mut v)=data.remove("unary"){
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("unary_functions"){
if !v.is_null() {
v=decode_Latexmath_Model_UnaryFunctions(v)?;
}
data.insert("unary_functions".into(),v);}
if let Some(mut v)=data.remove("underover"){
data.insert("underover".into(),v);}
if let Some(mut v)=data.remove("unicode_symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("unicode_symbols".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_Power(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::Power@default")?;let mut wire=Map::new();
for mut v in data.remove("binary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Binary(v)?;
}
wire.insert("binary".into(),v);}
for mut v in data.remove("binary_functions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_BinaryFunctions(v)?;
}
wire.insert("binary_functions".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("environment").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Environment(v)?;
}
wire.insert("environment".into(),v);}
for mut v in data.remove("expression").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Expression(v)?;
}
wire.insert("expression".into(),v);}
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_FirstValue(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("left_right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_LeftRight(v)?;
}
wire.insert("left_right".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("numeric_values").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("numeric_values".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("operant").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("operant".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("row_break").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("row_break".into(),v);}
for mut v in data.remove("space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("space".into(),v);}
for mut v in data.remove("subscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Subscript(v)?;
}
wire.insert("subscript".into(),v);}
for mut v in data.remove("supscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Supscript(v)?;
}
wire.insert("supscript".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SymbolNode(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("symbols".into(),v);}
for mut v in data.remove("ternary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TernaryClass(v)?;
}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("text".into(),v);}
for mut v in data.remove("three_per_em_space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("three_per_em_space".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("unary".into(),v);}
for mut v in data.remove("unary_functions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnaryFunctions(v)?;
}
wire.insert("unary_functions".into(),v);}
for mut v in data.remove("underover").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("underover".into(),v);}
for mut v in data.remove("unicode_symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("unicode_symbols".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_PowerBase_json(input:&str)->ContractResult<Latexmath_Model_PowerBase>{let data=decode_Latexmath_Model_PowerBase(parse_json(input,"Latexmath::Model::PowerBase@default")?)?; let model:Latexmath_Model_PowerBase=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_PowerBase_json(model:&Latexmath_Model_PowerBase)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_PowerBase(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_PowerBase(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::PowerBase@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("binary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "binary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("environment") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "environment", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("environment".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expression") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "expression", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expression".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("numeric_values") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "numeric_values", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("numeric_values".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("operant") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "operant", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("operant".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("row_break") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "row_break", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("row_break".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("subscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "subscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("subscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("supscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "supscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("supscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbols".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "ternary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("three_per_em_space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "three_per_em_space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("three_per_em_space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_functions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "unary_functions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_functions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("underover") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "underover", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("underover".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unicode_symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "unicode_symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unicode_symbols".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::PowerBase@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_PowerBase(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_PowerBase(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("binary"){
if !v.is_null() {
v=decode_Latexmath_Model_Binary(v)?;
}
data.insert("binary".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("environment"){
if !v.is_null() {
v=decode_Latexmath_Model_Environment(v)?;
}
data.insert("environment".into(),v);}
if let Some(mut v)=data.remove("expression"){
if !v.is_null() {
v=decode_Latexmath_Model_Expression(v)?;
}
data.insert("expression".into(),v);}
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Latexmath_Model_FirstValue(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("fonts"){
data.insert("fonts".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Latexmath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("numeric_values"){
data.insert("numeric_values".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("operant"){
data.insert("operant".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Latexmath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("row_break"){
data.insert("row_break".into(),v);}
if let Some(mut v)=data.remove("space"){
data.insert("space".into(),v);}
if let Some(mut v)=data.remove("subscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Subscript(v)?;
}
data.insert("subscript".into(),v);}
if let Some(mut v)=data.remove("supscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Supscript(v)?;
}
data.insert("supscript".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Latexmath_Model_SymbolNode(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("symbols".into(),v);}
if let Some(mut v)=data.remove("ternary"){
data.insert("ternary".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
if !v.is_null() {
v=decode_Latexmath_Model_TernaryClass(v)?;
}
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("three_per_em_space"){
data.insert("three_per_em_space".into(),v);}
if let Some(mut v)=data.remove("unary"){
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("unary_functions"){
if !v.is_null() {
v=decode_Latexmath_Model_UnaryFunctions(v)?;
}
data.insert("unary_functions".into(),v);}
if let Some(mut v)=data.remove("underover"){
data.insert("underover".into(),v);}
if let Some(mut v)=data.remove("unicode_symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("unicode_symbols".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_PowerBase(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::PowerBase@default")?;let mut wire=Map::new();
for mut v in data.remove("binary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Binary(v)?;
}
wire.insert("binary".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("environment").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Environment(v)?;
}
wire.insert("environment".into(),v);}
for mut v in data.remove("expression").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Expression(v)?;
}
wire.insert("expression".into(),v);}
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_FirstValue(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("numeric_values").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("numeric_values".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("operant").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("operant".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("row_break").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("row_break".into(),v);}
for mut v in data.remove("space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("space".into(),v);}
for mut v in data.remove("subscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Subscript(v)?;
}
wire.insert("subscript".into(),v);}
for mut v in data.remove("supscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Supscript(v)?;
}
wire.insert("supscript".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SymbolNode(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("symbols".into(),v);}
for mut v in data.remove("ternary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TernaryClass(v)?;
}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("text".into(),v);}
for mut v in data.remove("three_per_em_space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("three_per_em_space".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("unary".into(),v);}
for mut v in data.remove("unary_functions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnaryFunctions(v)?;
}
wire.insert("unary_functions".into(),v);}
for mut v in data.remove("underover").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("underover".into(),v);}
for mut v in data.remove("unicode_symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("unicode_symbols".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_RightParen_json(input:&str)->ContractResult<Latexmath_Model_RightParen>{let data=decode_Latexmath_Model_RightParen(parse_json(input,"Latexmath::Model::RightParen@default")?)?; let model:Latexmath_Model_RightParen=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::RightParen@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_RightParen_json(model:&Latexmath_Model_RightParen)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::RightParen@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_RightParen(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::RightParen@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_RightParen(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::RightParen@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("absent") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::RightParen@default", "absent", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("absent".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("literal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::RightParen@default", "literal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("rparen") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::RightParen@default", "rparen", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("rparen".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::RightParen@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_RightParen(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_RightParen(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("absent"){
data.insert("absent".into(),v);}
if let Some(mut v)=data.remove("literal"){
data.insert("literal".into(),v);}
if let Some(mut v)=data.remove("rparen"){
data.insert("rparen".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_RightParen(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::RightParen@default")?;let mut wire=Map::new();
for mut v in data.remove("absent").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("absent".into(),v);}
for mut v in data.remove("literal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal".into(),v);}
for mut v in data.remove("rparen").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("rparen".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_SecondValue_json(input:&str)->ContractResult<Latexmath_Model_SecondValue>{let data=decode_Latexmath_Model_SecondValue(parse_json(input,"Latexmath::Model::SecondValue@default")?)?; let model:Latexmath_Model_SecondValue=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_SecondValue_json(model:&Latexmath_Model_SecondValue)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_SecondValue(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_SecondValue(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::SecondValue@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("absent") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "absent", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("absent".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "binary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("dividend") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "dividend", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("dividend".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("divisor") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "divisor", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("divisor".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("environment") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "environment", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("environment".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expression") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "expression", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expression".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left_right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "left_right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left_right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("limits") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "limits", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("limits".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("numeric_values") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "numeric_values", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("numeric_values".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("operant") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "operant", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("operant".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("over") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "over", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("over".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "power", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("row_break") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "row_break", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("row_break".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("second_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "second_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("second_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sequence") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "sequence", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sequence".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("subscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "subscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("subscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("substack") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "substack", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("substack".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("supscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "supscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("supscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbols".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table_data") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "table_data", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table_data".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "ternary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("three_per_em_space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "three_per_em_space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("three_per_em_space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_functions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "unary_functions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_functions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("under_over") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "under_over", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("under_over".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("underover") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "underover", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("underover".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unicode_symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "unicode_symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unicode_symbols".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SecondValue@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_SecondValue(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_SecondValue(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("absent"){
data.insert("absent".into(),v);}
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Latexmath_Model_Base(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("binary"){
if !v.is_null() {
v=decode_Latexmath_Model_Binary(v)?;
}
data.insert("binary".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("dividend"){
if !v.is_null() {
v=decode_Latexmath_Model_Dividend(v)?;
}
data.insert("dividend".into(),v);}
if let Some(mut v)=data.remove("divisor"){
if !v.is_null() {
v=decode_Latexmath_Model_Divisor(v)?;
}
data.insert("divisor".into(),v);}
if let Some(mut v)=data.remove("environment"){
if !v.is_null() {
v=decode_Latexmath_Model_Environment(v)?;
}
data.insert("environment".into(),v);}
if let Some(mut v)=data.remove("expression"){
if !v.is_null() {
v=decode_Latexmath_Model_Expression(v)?;
}
data.insert("expression".into(),v);}
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Latexmath_Model_FirstValue(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("fonts"){
data.insert("fonts".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Latexmath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("left_right"){
if !v.is_null() {
v=decode_Latexmath_Model_LeftRight(v)?;
}
data.insert("left_right".into(),v);}
if let Some(mut v)=data.remove("limits"){
if !v.is_null() {
v=decode_Latexmath_Model_Limits(v)?;
}
data.insert("limits".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("numeric_values"){
data.insert("numeric_values".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("operant"){
data.insert("operant".into(),v);}
if let Some(mut v)=data.remove("over"){
if !v.is_null() {
v=decode_Latexmath_Model_Over(v)?;
}
data.insert("over".into(),v);}
if let Some(mut v)=data.remove("power"){
if !v.is_null() {
v=decode_Latexmath_Model_Power(v)?;
}
data.insert("power".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Latexmath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("row_break"){
data.insert("row_break".into(),v);}
if let Some(mut v)=data.remove("second_value"){
if !v.is_null() {
v=decode_Latexmath_Model_SecondValue(v)?;
}
data.insert("second_value".into(),v);}
if let Some(mut v)=data.remove("sequence"){
if !v.is_null() {
v=decode_Latexmath_Model_Sequence(v)?;
}
data.insert("sequence".into(),v);}
if let Some(mut v)=data.remove("space"){
data.insert("space".into(),v);}
if let Some(mut v)=data.remove("subscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Subscript(v)?;
}
data.insert("subscript".into(),v);}
if let Some(mut v)=data.remove("substack"){
data.insert("substack".into(),v);}
if let Some(mut v)=data.remove("supscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Supscript(v)?;
}
data.insert("supscript".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Latexmath_Model_SymbolNode(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("symbols".into(),v);}
if let Some(mut v)=data.remove("table_data"){
if !v.is_null() {
v=decode_Latexmath_Model_TableData(v)?;
}
data.insert("table_data".into(),v);}
if let Some(mut v)=data.remove("ternary"){
data.insert("ternary".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
if !v.is_null() {
v=decode_Latexmath_Model_TernaryClass(v)?;
}
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("three_per_em_space"){
data.insert("three_per_em_space".into(),v);}
if let Some(mut v)=data.remove("unary"){
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("unary_functions"){
if !v.is_null() {
v=decode_Latexmath_Model_UnaryFunctions(v)?;
}
data.insert("unary_functions".into(),v);}
if let Some(mut v)=data.remove("under_over"){
if !v.is_null() {
v=decode_Latexmath_Model_UnderOver(v)?;
}
data.insert("under_over".into(),v);}
if let Some(mut v)=data.remove("underover"){
data.insert("underover".into(),v);}
if let Some(mut v)=data.remove("unicode_symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("unicode_symbols".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_SecondValue(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::SecondValue@default")?;let mut wire=Map::new();
for mut v in data.remove("absent").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("absent".into(),v);}
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Base(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("binary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Binary(v)?;
}
wire.insert("binary".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("dividend").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Dividend(v)?;
}
wire.insert("dividend".into(),v);}
for mut v in data.remove("divisor").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Divisor(v)?;
}
wire.insert("divisor".into(),v);}
for mut v in data.remove("environment").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Environment(v)?;
}
wire.insert("environment".into(),v);}
for mut v in data.remove("expression").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Expression(v)?;
}
wire.insert("expression".into(),v);}
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_FirstValue(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("left_right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_LeftRight(v)?;
}
wire.insert("left_right".into(),v);}
for mut v in data.remove("limits").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Limits(v)?;
}
wire.insert("limits".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("numeric_values").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("numeric_values".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("operant").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("operant".into(),v);}
for mut v in data.remove("over").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Over(v)?;
}
wire.insert("over".into(),v);}
for mut v in data.remove("power").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Power(v)?;
}
wire.insert("power".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("row_break").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("row_break".into(),v);}
for mut v in data.remove("second_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SecondValue(v)?;
}
wire.insert("second_value".into(),v);}
for mut v in data.remove("sequence").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Sequence(v)?;
}
wire.insert("sequence".into(),v);}
for mut v in data.remove("space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("space".into(),v);}
for mut v in data.remove("subscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Subscript(v)?;
}
wire.insert("subscript".into(),v);}
for mut v in data.remove("substack").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("substack".into(),v);}
for mut v in data.remove("supscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Supscript(v)?;
}
wire.insert("supscript".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SymbolNode(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("symbols".into(),v);}
for mut v in data.remove("table_data").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TableData(v)?;
}
wire.insert("table_data".into(),v);}
for mut v in data.remove("ternary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TernaryClass(v)?;
}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("text".into(),v);}
for mut v in data.remove("three_per_em_space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("three_per_em_space".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("unary".into(),v);}
for mut v in data.remove("unary_functions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnaryFunctions(v)?;
}
wire.insert("unary_functions".into(),v);}
for mut v in data.remove("under_over").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnderOver(v)?;
}
wire.insert("under_over".into(),v);}
for mut v in data.remove("underover").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("underover".into(),v);}
for mut v in data.remove("unicode_symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("unicode_symbols".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_Sequence_json(input:&str)->ContractResult<Latexmath_Model_Sequence>{let data=decode_Latexmath_Model_Sequence(parse_json(input,"Latexmath::Model::Sequence@default")?)?; let model:Latexmath_Model_Sequence=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_Sequence_json(model:&Latexmath_Model_Sequence)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_Sequence(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_Sequence(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::Sequence@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "binary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("environment") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "environment", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("environment".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expression") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "expression", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expression".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left_right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "left_right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left_right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("limits") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "limits", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("limits".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("numeric_values") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "numeric_values", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("numeric_values".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("operant") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "operant", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("operant".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("over") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "over", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("over".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "power", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("row_break") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "row_break", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("row_break".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("second_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "second_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("second_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("subscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "subscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("subscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("substack") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "substack", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("substack".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("supscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "supscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("supscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbols".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table_data") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "table_data", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table_data".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "ternary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("three_per_em_space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "three_per_em_space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("three_per_em_space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_functions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "unary_functions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_functions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("under_over") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "under_over", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("under_over".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("underover") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "underover", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("underover".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unicode_symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "unicode_symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unicode_symbols".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Sequence@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_Sequence(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_Sequence(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Latexmath_Model_Base(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("binary"){
if !v.is_null() {
v=decode_Latexmath_Model_Binary(v)?;
}
data.insert("binary".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("environment"){
if !v.is_null() {
v=decode_Latexmath_Model_Environment(v)?;
}
data.insert("environment".into(),v);}
if let Some(mut v)=data.remove("expression"){
if !v.is_null() {
v=decode_Latexmath_Model_Expression(v)?;
}
data.insert("expression".into(),v);}
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Latexmath_Model_FirstValue(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("fonts"){
data.insert("fonts".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Latexmath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("left_right"){
if !v.is_null() {
v=decode_Latexmath_Model_LeftRight(v)?;
}
data.insert("left_right".into(),v);}
if let Some(mut v)=data.remove("limits"){
if !v.is_null() {
v=decode_Latexmath_Model_Limits(v)?;
}
data.insert("limits".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("numeric_values"){
data.insert("numeric_values".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("operant"){
data.insert("operant".into(),v);}
if let Some(mut v)=data.remove("over"){
if !v.is_null() {
v=decode_Latexmath_Model_Over(v)?;
}
data.insert("over".into(),v);}
if let Some(mut v)=data.remove("power"){
if !v.is_null() {
v=decode_Latexmath_Model_Power(v)?;
}
data.insert("power".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Latexmath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("row_break"){
data.insert("row_break".into(),v);}
if let Some(mut v)=data.remove("second_value"){
if !v.is_null() {
v=decode_Latexmath_Model_SecondValue(v)?;
}
data.insert("second_value".into(),v);}
if let Some(mut v)=data.remove("space"){
data.insert("space".into(),v);}
if let Some(mut v)=data.remove("subscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Subscript(v)?;
}
data.insert("subscript".into(),v);}
if let Some(mut v)=data.remove("substack"){
data.insert("substack".into(),v);}
if let Some(mut v)=data.remove("supscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Supscript(v)?;
}
data.insert("supscript".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Latexmath_Model_SymbolNode(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("symbols".into(),v);}
if let Some(mut v)=data.remove("table_data"){
if !v.is_null() {
v=decode_Latexmath_Model_TableData(v)?;
}
data.insert("table_data".into(),v);}
if let Some(mut v)=data.remove("ternary"){
data.insert("ternary".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
if !v.is_null() {
v=decode_Latexmath_Model_TernaryClass(v)?;
}
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("three_per_em_space"){
data.insert("three_per_em_space".into(),v);}
if let Some(mut v)=data.remove("unary"){
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("unary_functions"){
if !v.is_null() {
v=decode_Latexmath_Model_UnaryFunctions(v)?;
}
data.insert("unary_functions".into(),v);}
if let Some(mut v)=data.remove("under_over"){
if !v.is_null() {
v=decode_Latexmath_Model_UnderOver(v)?;
}
data.insert("under_over".into(),v);}
if let Some(mut v)=data.remove("underover"){
data.insert("underover".into(),v);}
if let Some(mut v)=data.remove("unicode_symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("unicode_symbols".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_Sequence(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::Sequence@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Base(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("binary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Binary(v)?;
}
wire.insert("binary".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("environment").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Environment(v)?;
}
wire.insert("environment".into(),v);}
for mut v in data.remove("expression").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Expression(v)?;
}
wire.insert("expression".into(),v);}
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_FirstValue(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("left_right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_LeftRight(v)?;
}
wire.insert("left_right".into(),v);}
for mut v in data.remove("limits").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Limits(v)?;
}
wire.insert("limits".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("numeric_values").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("numeric_values".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("operant").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("operant".into(),v);}
for mut v in data.remove("over").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Over(v)?;
}
wire.insert("over".into(),v);}
for mut v in data.remove("power").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Power(v)?;
}
wire.insert("power".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("row_break").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("row_break".into(),v);}
for mut v in data.remove("second_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SecondValue(v)?;
}
wire.insert("second_value".into(),v);}
for mut v in data.remove("space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("space".into(),v);}
for mut v in data.remove("subscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Subscript(v)?;
}
wire.insert("subscript".into(),v);}
for mut v in data.remove("substack").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("substack".into(),v);}
for mut v in data.remove("supscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Supscript(v)?;
}
wire.insert("supscript".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SymbolNode(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("symbols".into(),v);}
for mut v in data.remove("table_data").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TableData(v)?;
}
wire.insert("table_data".into(),v);}
for mut v in data.remove("ternary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TernaryClass(v)?;
}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("text".into(),v);}
for mut v in data.remove("three_per_em_space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("three_per_em_space".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("unary".into(),v);}
for mut v in data.remove("unary_functions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnaryFunctions(v)?;
}
wire.insert("unary_functions".into(),v);}
for mut v in data.remove("under_over").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnderOver(v)?;
}
wire.insert("under_over".into(),v);}
for mut v in data.remove("underover").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("underover".into(),v);}
for mut v in data.remove("unicode_symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("unicode_symbols".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_Subscript_json(input:&str)->ContractResult<Latexmath_Model_Subscript>{let data=decode_Latexmath_Model_Subscript(parse_json(input,"Latexmath::Model::Subscript@default")?)?; let model:Latexmath_Model_Subscript=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_Subscript_json(model:&Latexmath_Model_Subscript)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_Subscript(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_Subscript(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::Subscript@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "binary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("environment") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "environment", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("environment".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expression") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "expression", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expression".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left_right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "left_right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left_right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("limits") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "limits", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("limits".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("numeric_values") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "numeric_values", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("numeric_values".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("operant") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "operant", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("operant".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("over") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "over", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("over".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "power", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("row_break") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "row_break", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("row_break".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("second_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "second_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("second_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("subscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "subscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("subscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("substack") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "substack", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("substack".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("supscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "supscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("supscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbols".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table_data") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "table_data", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table_data".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "ternary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("three_per_em_space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "three_per_em_space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("three_per_em_space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_functions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "unary_functions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_functions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("under_over") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "under_over", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("under_over".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("underover") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "underover", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("underover".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unicode_symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "unicode_symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unicode_symbols".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Subscript@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_Subscript(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_Subscript(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Latexmath_Model_Base(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("binary"){
if !v.is_null() {
v=decode_Latexmath_Model_Binary(v)?;
}
data.insert("binary".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("environment"){
if !v.is_null() {
v=decode_Latexmath_Model_Environment(v)?;
}
data.insert("environment".into(),v);}
if let Some(mut v)=data.remove("expression"){
if !v.is_null() {
v=decode_Latexmath_Model_Expression(v)?;
}
data.insert("expression".into(),v);}
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Latexmath_Model_FirstValue(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("fonts"){
data.insert("fonts".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Latexmath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("left_right"){
if !v.is_null() {
v=decode_Latexmath_Model_LeftRight(v)?;
}
data.insert("left_right".into(),v);}
if let Some(mut v)=data.remove("limits"){
if !v.is_null() {
v=decode_Latexmath_Model_Limits(v)?;
}
data.insert("limits".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("numeric_values"){
data.insert("numeric_values".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("operant"){
data.insert("operant".into(),v);}
if let Some(mut v)=data.remove("over"){
if !v.is_null() {
v=decode_Latexmath_Model_Over(v)?;
}
data.insert("over".into(),v);}
if let Some(mut v)=data.remove("power"){
if !v.is_null() {
v=decode_Latexmath_Model_Power(v)?;
}
data.insert("power".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Latexmath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("row_break"){
data.insert("row_break".into(),v);}
if let Some(mut v)=data.remove("second_value"){
if !v.is_null() {
v=decode_Latexmath_Model_SecondValue(v)?;
}
data.insert("second_value".into(),v);}
if let Some(mut v)=data.remove("space"){
data.insert("space".into(),v);}
if let Some(mut v)=data.remove("subscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Subscript(v)?;
}
data.insert("subscript".into(),v);}
if let Some(mut v)=data.remove("substack"){
data.insert("substack".into(),v);}
if let Some(mut v)=data.remove("supscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Supscript(v)?;
}
data.insert("supscript".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Latexmath_Model_SymbolNode(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("symbols".into(),v);}
if let Some(mut v)=data.remove("table_data"){
if !v.is_null() {
v=decode_Latexmath_Model_TableData(v)?;
}
data.insert("table_data".into(),v);}
if let Some(mut v)=data.remove("ternary"){
data.insert("ternary".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
if !v.is_null() {
v=decode_Latexmath_Model_TernaryClass(v)?;
}
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("three_per_em_space"){
data.insert("three_per_em_space".into(),v);}
if let Some(mut v)=data.remove("unary"){
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("unary_functions"){
if !v.is_null() {
v=decode_Latexmath_Model_UnaryFunctions(v)?;
}
data.insert("unary_functions".into(),v);}
if let Some(mut v)=data.remove("under_over"){
if !v.is_null() {
v=decode_Latexmath_Model_UnderOver(v)?;
}
data.insert("under_over".into(),v);}
if let Some(mut v)=data.remove("underover"){
data.insert("underover".into(),v);}
if let Some(mut v)=data.remove("unicode_symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("unicode_symbols".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_Subscript(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::Subscript@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Base(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("binary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Binary(v)?;
}
wire.insert("binary".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("environment").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Environment(v)?;
}
wire.insert("environment".into(),v);}
for mut v in data.remove("expression").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Expression(v)?;
}
wire.insert("expression".into(),v);}
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_FirstValue(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("left_right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_LeftRight(v)?;
}
wire.insert("left_right".into(),v);}
for mut v in data.remove("limits").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Limits(v)?;
}
wire.insert("limits".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("numeric_values").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("numeric_values".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("operant").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("operant".into(),v);}
for mut v in data.remove("over").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Over(v)?;
}
wire.insert("over".into(),v);}
for mut v in data.remove("power").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Power(v)?;
}
wire.insert("power".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("row_break").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("row_break".into(),v);}
for mut v in data.remove("second_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SecondValue(v)?;
}
wire.insert("second_value".into(),v);}
for mut v in data.remove("space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("space".into(),v);}
for mut v in data.remove("subscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Subscript(v)?;
}
wire.insert("subscript".into(),v);}
for mut v in data.remove("substack").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("substack".into(),v);}
for mut v in data.remove("supscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Supscript(v)?;
}
wire.insert("supscript".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SymbolNode(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("symbols".into(),v);}
for mut v in data.remove("table_data").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TableData(v)?;
}
wire.insert("table_data".into(),v);}
for mut v in data.remove("ternary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TernaryClass(v)?;
}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("text".into(),v);}
for mut v in data.remove("three_per_em_space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("three_per_em_space".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("unary".into(),v);}
for mut v in data.remove("unary_functions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnaryFunctions(v)?;
}
wire.insert("unary_functions".into(),v);}
for mut v in data.remove("under_over").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnderOver(v)?;
}
wire.insert("under_over".into(),v);}
for mut v in data.remove("underover").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("underover".into(),v);}
for mut v in data.remove("unicode_symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("unicode_symbols".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_Supscript_json(input:&str)->ContractResult<Latexmath_Model_Supscript>{let data=decode_Latexmath_Model_Supscript(parse_json(input,"Latexmath::Model::Supscript@default")?)?; let model:Latexmath_Model_Supscript=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_Supscript_json(model:&Latexmath_Model_Supscript)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_Supscript(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_Supscript(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::Supscript@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "binary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("environment") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "environment", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("environment".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expression") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "expression", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expression".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left_right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "left_right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left_right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("limits") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "limits", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("limits".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("numeric_values") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "numeric_values", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("numeric_values".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("operant") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "operant", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("operant".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("over") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "over", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("over".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "power", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("row_break") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "row_break", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("row_break".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("second_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "second_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("second_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("subscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "subscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("subscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("substack") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "substack", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("substack".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("supscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "supscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("supscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbols".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table_data") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "table_data", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table_data".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "ternary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("three_per_em_space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "three_per_em_space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("three_per_em_space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_functions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "unary_functions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_functions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("under_over") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "under_over", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("under_over".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("underover") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "underover", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("underover".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unicode_symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "unicode_symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unicode_symbols".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::Supscript@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_Supscript(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_Supscript(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Latexmath_Model_Base(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("binary"){
if !v.is_null() {
v=decode_Latexmath_Model_Binary(v)?;
}
data.insert("binary".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("environment"){
if !v.is_null() {
v=decode_Latexmath_Model_Environment(v)?;
}
data.insert("environment".into(),v);}
if let Some(mut v)=data.remove("expression"){
if !v.is_null() {
v=decode_Latexmath_Model_Expression(v)?;
}
data.insert("expression".into(),v);}
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Latexmath_Model_FirstValue(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("fonts"){
data.insert("fonts".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Latexmath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("left_right"){
if !v.is_null() {
v=decode_Latexmath_Model_LeftRight(v)?;
}
data.insert("left_right".into(),v);}
if let Some(mut v)=data.remove("limits"){
if !v.is_null() {
v=decode_Latexmath_Model_Limits(v)?;
}
data.insert("limits".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("numeric_values"){
data.insert("numeric_values".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("operant"){
data.insert("operant".into(),v);}
if let Some(mut v)=data.remove("over"){
if !v.is_null() {
v=decode_Latexmath_Model_Over(v)?;
}
data.insert("over".into(),v);}
if let Some(mut v)=data.remove("power"){
if !v.is_null() {
v=decode_Latexmath_Model_Power(v)?;
}
data.insert("power".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Latexmath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("row_break"){
data.insert("row_break".into(),v);}
if let Some(mut v)=data.remove("second_value"){
if !v.is_null() {
v=decode_Latexmath_Model_SecondValue(v)?;
}
data.insert("second_value".into(),v);}
if let Some(mut v)=data.remove("space"){
data.insert("space".into(),v);}
if let Some(mut v)=data.remove("subscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Subscript(v)?;
}
data.insert("subscript".into(),v);}
if let Some(mut v)=data.remove("substack"){
data.insert("substack".into(),v);}
if let Some(mut v)=data.remove("supscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Supscript(v)?;
}
data.insert("supscript".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Latexmath_Model_SymbolNode(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("symbols".into(),v);}
if let Some(mut v)=data.remove("table_data"){
if !v.is_null() {
v=decode_Latexmath_Model_TableData(v)?;
}
data.insert("table_data".into(),v);}
if let Some(mut v)=data.remove("ternary"){
data.insert("ternary".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
if !v.is_null() {
v=decode_Latexmath_Model_TernaryClass(v)?;
}
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("three_per_em_space"){
data.insert("three_per_em_space".into(),v);}
if let Some(mut v)=data.remove("unary"){
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("unary_functions"){
if !v.is_null() {
v=decode_Latexmath_Model_UnaryFunctions(v)?;
}
data.insert("unary_functions".into(),v);}
if let Some(mut v)=data.remove("under_over"){
if !v.is_null() {
v=decode_Latexmath_Model_UnderOver(v)?;
}
data.insert("under_over".into(),v);}
if let Some(mut v)=data.remove("underover"){
data.insert("underover".into(),v);}
if let Some(mut v)=data.remove("unicode_symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("unicode_symbols".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_Supscript(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::Supscript@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Base(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("binary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Binary(v)?;
}
wire.insert("binary".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("environment").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Environment(v)?;
}
wire.insert("environment".into(),v);}
for mut v in data.remove("expression").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Expression(v)?;
}
wire.insert("expression".into(),v);}
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_FirstValue(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("left_right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_LeftRight(v)?;
}
wire.insert("left_right".into(),v);}
for mut v in data.remove("limits").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Limits(v)?;
}
wire.insert("limits".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("numeric_values").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("numeric_values".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("operant").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("operant".into(),v);}
for mut v in data.remove("over").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Over(v)?;
}
wire.insert("over".into(),v);}
for mut v in data.remove("power").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Power(v)?;
}
wire.insert("power".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("row_break").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("row_break".into(),v);}
for mut v in data.remove("second_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SecondValue(v)?;
}
wire.insert("second_value".into(),v);}
for mut v in data.remove("space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("space".into(),v);}
for mut v in data.remove("subscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Subscript(v)?;
}
wire.insert("subscript".into(),v);}
for mut v in data.remove("substack").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("substack".into(),v);}
for mut v in data.remove("supscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Supscript(v)?;
}
wire.insert("supscript".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SymbolNode(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("symbols".into(),v);}
for mut v in data.remove("table_data").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TableData(v)?;
}
wire.insert("table_data".into(),v);}
for mut v in data.remove("ternary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TernaryClass(v)?;
}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("text".into(),v);}
for mut v in data.remove("three_per_em_space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("three_per_em_space".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("unary".into(),v);}
for mut v in data.remove("unary_functions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnaryFunctions(v)?;
}
wire.insert("unary_functions".into(),v);}
for mut v in data.remove("under_over").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnderOver(v)?;
}
wire.insert("under_over".into(),v);}
for mut v in data.remove("underover").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("underover".into(),v);}
for mut v in data.remove("unicode_symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("unicode_symbols".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_SymbolNode_json(input:&str)->ContractResult<Latexmath_Model_SymbolNode>{let data=decode_Latexmath_Model_SymbolNode(parse_json(input,"Latexmath::Model::SymbolNode@default")?)?; let model:Latexmath_Model_SymbolNode=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::SymbolNode@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_SymbolNode_json(model:&Latexmath_Model_SymbolNode)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::SymbolNode@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_SymbolNode(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::SymbolNode@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_SymbolNode(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::SymbolNode@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("literal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SymbolNode@default", "literal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("rparen") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SymbolNode@default", "rparen", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("rparen".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text_list") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SymbolNode@default", "text_list", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text_list".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::SymbolNode@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_SymbolNode(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_SymbolNode(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("literal"){
data.insert("literal".into(),v);}
if let Some(mut v)=data.remove("rparen"){
data.insert("rparen".into(),v);}
if let Some(mut v)=data.remove("text_list"){
data.insert("text_list".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_SymbolNode(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::SymbolNode@default")?;let mut wire=Map::new();
for mut v in data.remove("literal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal".into(),v);}
for mut v in data.remove("rparen").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("rparen".into(),v);}
for mut v in data.remove("text_list").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("text_list".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_TableData_json(input:&str)->ContractResult<Latexmath_Model_TableData>{let data=decode_Latexmath_Model_TableData(parse_json(input,"Latexmath::Model::TableData@default")?)?; let model:Latexmath_Model_TableData=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_TableData_json(model:&Latexmath_Model_TableData)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_TableData(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_TableData(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::TableData@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "binary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("dividend") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "dividend", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("dividend".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("divisor") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "divisor", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("divisor".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("environment") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "environment", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("environment".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expression") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "expression", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expression".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left_right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "left_right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left_right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("limits") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "limits", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("limits".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("numeric_values") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "numeric_values", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("numeric_values".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("operant") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "operant", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("operant".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("over") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "over", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("over".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "power", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("row_break") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "row_break", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("row_break".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("second_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "second_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("second_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("sequence") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "sequence", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("sequence".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("subscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "subscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("subscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("substack") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "substack", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("substack".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("supscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "supscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("supscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbols".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table_data") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "table_data", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table_data".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "ternary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("three_per_em_space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "three_per_em_space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("three_per_em_space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_functions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "unary_functions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_functions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("under_over") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "under_over", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("under_over".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("underover") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "underover", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("underover".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unicode_symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "unicode_symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unicode_symbols".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TableData@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_TableData(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_TableData(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Latexmath_Model_Base(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("binary"){
if !v.is_null() {
v=decode_Latexmath_Model_Binary(v)?;
}
data.insert("binary".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("dividend"){
if !v.is_null() {
v=decode_Latexmath_Model_Dividend(v)?;
}
data.insert("dividend".into(),v);}
if let Some(mut v)=data.remove("divisor"){
if !v.is_null() {
v=decode_Latexmath_Model_Divisor(v)?;
}
data.insert("divisor".into(),v);}
if let Some(mut v)=data.remove("environment"){
if !v.is_null() {
v=decode_Latexmath_Model_Environment(v)?;
}
data.insert("environment".into(),v);}
if let Some(mut v)=data.remove("expression"){
if !v.is_null() {
v=decode_Latexmath_Model_Expression(v)?;
}
data.insert("expression".into(),v);}
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Latexmath_Model_FirstValue(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("fonts"){
data.insert("fonts".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Latexmath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("left_right"){
if !v.is_null() {
v=decode_Latexmath_Model_LeftRight(v)?;
}
data.insert("left_right".into(),v);}
if let Some(mut v)=data.remove("limits"){
if !v.is_null() {
v=decode_Latexmath_Model_Limits(v)?;
}
data.insert("limits".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("numeric_values"){
data.insert("numeric_values".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("operant"){
data.insert("operant".into(),v);}
if let Some(mut v)=data.remove("over"){
if !v.is_null() {
v=decode_Latexmath_Model_Over(v)?;
}
data.insert("over".into(),v);}
if let Some(mut v)=data.remove("power"){
if !v.is_null() {
v=decode_Latexmath_Model_Power(v)?;
}
data.insert("power".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Latexmath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("row_break"){
data.insert("row_break".into(),v);}
if let Some(mut v)=data.remove("second_value"){
if !v.is_null() {
v=decode_Latexmath_Model_SecondValue(v)?;
}
data.insert("second_value".into(),v);}
if let Some(mut v)=data.remove("sequence"){
if !v.is_null() {
v=decode_Latexmath_Model_Sequence(v)?;
}
data.insert("sequence".into(),v);}
if let Some(mut v)=data.remove("space"){
data.insert("space".into(),v);}
if let Some(mut v)=data.remove("subscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Subscript(v)?;
}
data.insert("subscript".into(),v);}
if let Some(mut v)=data.remove("substack"){
data.insert("substack".into(),v);}
if let Some(mut v)=data.remove("supscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Supscript(v)?;
}
data.insert("supscript".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Latexmath_Model_SymbolNode(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("symbols".into(),v);}
if let Some(mut v)=data.remove("table_data"){
if !v.is_null() {
v=decode_Latexmath_Model_TableData(v)?;
}
data.insert("table_data".into(),v);}
if let Some(mut v)=data.remove("ternary"){
data.insert("ternary".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
if !v.is_null() {
v=decode_Latexmath_Model_TernaryClass(v)?;
}
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("three_per_em_space"){
data.insert("three_per_em_space".into(),v);}
if let Some(mut v)=data.remove("unary"){
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("unary_functions"){
if !v.is_null() {
v=decode_Latexmath_Model_UnaryFunctions(v)?;
}
data.insert("unary_functions".into(),v);}
if let Some(mut v)=data.remove("under_over"){
if !v.is_null() {
v=decode_Latexmath_Model_UnderOver(v)?;
}
data.insert("under_over".into(),v);}
if let Some(mut v)=data.remove("underover"){
data.insert("underover".into(),v);}
if let Some(mut v)=data.remove("unicode_symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("unicode_symbols".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_TableData(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::TableData@default")?;let mut wire=Map::new();
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Base(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("binary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Binary(v)?;
}
wire.insert("binary".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("dividend").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Dividend(v)?;
}
wire.insert("dividend".into(),v);}
for mut v in data.remove("divisor").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Divisor(v)?;
}
wire.insert("divisor".into(),v);}
for mut v in data.remove("environment").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Environment(v)?;
}
wire.insert("environment".into(),v);}
for mut v in data.remove("expression").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Expression(v)?;
}
wire.insert("expression".into(),v);}
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_FirstValue(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("left_right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_LeftRight(v)?;
}
wire.insert("left_right".into(),v);}
for mut v in data.remove("limits").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Limits(v)?;
}
wire.insert("limits".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("numeric_values").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("numeric_values".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("operant").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("operant".into(),v);}
for mut v in data.remove("over").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Over(v)?;
}
wire.insert("over".into(),v);}
for mut v in data.remove("power").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Power(v)?;
}
wire.insert("power".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("row_break").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("row_break".into(),v);}
for mut v in data.remove("second_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SecondValue(v)?;
}
wire.insert("second_value".into(),v);}
for mut v in data.remove("sequence").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Sequence(v)?;
}
wire.insert("sequence".into(),v);}
for mut v in data.remove("space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("space".into(),v);}
for mut v in data.remove("subscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Subscript(v)?;
}
wire.insert("subscript".into(),v);}
for mut v in data.remove("substack").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("substack".into(),v);}
for mut v in data.remove("supscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Supscript(v)?;
}
wire.insert("supscript".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SymbolNode(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("symbols".into(),v);}
for mut v in data.remove("table_data").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TableData(v)?;
}
wire.insert("table_data".into(),v);}
for mut v in data.remove("ternary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TernaryClass(v)?;
}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("text".into(),v);}
for mut v in data.remove("three_per_em_space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("three_per_em_space".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("unary".into(),v);}
for mut v in data.remove("unary_functions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnaryFunctions(v)?;
}
wire.insert("unary_functions".into(),v);}
for mut v in data.remove("under_over").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnderOver(v)?;
}
wire.insert("under_over".into(),v);}
for mut v in data.remove("underover").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("underover".into(),v);}
for mut v in data.remove("unicode_symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("unicode_symbols".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_TernaryClass_json(input:&str)->ContractResult<Latexmath_Model_TernaryClass>{let data=decode_Latexmath_Model_TernaryClass(parse_json(input,"Latexmath::Model::TernaryClass@default")?)?; let model:Latexmath_Model_TernaryClass=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::TernaryClass@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_TernaryClass_json(model:&Latexmath_Model_TernaryClass)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::TernaryClass@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_TernaryClass(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::TernaryClass@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_TernaryClass(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::TernaryClass@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("subscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TernaryClass@default", "subscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("subscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("supscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TernaryClass@default", "supscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("supscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_functions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TernaryClass@default", "ternary_functions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_functions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("third_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TernaryClass@default", "third_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("third_value".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::TernaryClass@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_TernaryClass(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_TernaryClass(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("subscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Subscript(v)?;
}
data.insert("subscript".into(),v);}
if let Some(mut v)=data.remove("supscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Supscript(v)?;
}
data.insert("supscript".into(),v);}
if let Some(mut v)=data.remove("ternary_functions"){
data.insert("ternary_functions".into(),v);}
if let Some(mut v)=data.remove("third_value"){
if !v.is_null() {
v=decode_Latexmath_Model_ThirdValue(v)?;
}
data.insert("third_value".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_TernaryClass(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::TernaryClass@default")?;let mut wire=Map::new();
for mut v in data.remove("subscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Subscript(v)?;
}
wire.insert("subscript".into(),v);}
for mut v in data.remove("supscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Supscript(v)?;
}
wire.insert("supscript".into(),v);}
for mut v in data.remove("ternary_functions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary_functions".into(),v);}
for mut v in data.remove("third_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_ThirdValue(v)?;
}
wire.insert("third_value".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_ThirdValue_json(input:&str)->ContractResult<Latexmath_Model_ThirdValue>{let data=decode_Latexmath_Model_ThirdValue(parse_json(input,"Latexmath::Model::ThirdValue@default")?)?; let model:Latexmath_Model_ThirdValue=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_ThirdValue_json(model:&Latexmath_Model_ThirdValue)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_ThirdValue(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_ThirdValue(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::ThirdValue@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("absent") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "absent", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("absent".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "binary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("binary_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "binary_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("environment") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "environment", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("environment".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("expression") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "expression", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("expression".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("fonts") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "fonts", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("fonts".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("hex_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "hex_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("hex_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("intermediate_exp") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "intermediate_exp", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("intermediate_exp".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("left_right") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "left_right", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("left_right".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("limits") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "limits", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("limits".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("numeric_values") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "numeric_values", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("numeric_values".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("octal_number") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "octal_number", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("octal_number".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("operant") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "operant", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("operant".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("over") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "over", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("over".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "power", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("power_base") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "power_base", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("power_base".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("row_break") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "row_break", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("row_break".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("second_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "second_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("second_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("subscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "subscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("subscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("substack") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "substack", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("substack".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("supscript") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "supscript", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("supscript".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbol") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "symbol", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbol".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("symbols".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("table_data") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "table_data", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("table_data".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "ternary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("ternary_class") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "ternary_class", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("ternary_class".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("text") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "text", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("text".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("three_per_em_space") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "three_per_em_space", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("three_per_em_space".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary_functions") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "unary_functions", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary_functions".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("under_over") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "under_over", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("under_over".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("underover") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "underover", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("underover".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unicode_symbols") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "unicode_symbols", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unicode_symbols".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::ThirdValue@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_ThirdValue(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_ThirdValue(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("absent"){
data.insert("absent".into(),v);}
if let Some(mut v)=data.remove("base"){
if !v.is_null() {
v=decode_Latexmath_Model_Base(v)?;
}
data.insert("base".into(),v);}
if let Some(mut v)=data.remove("binary"){
if !v.is_null() {
v=decode_Latexmath_Model_Binary(v)?;
}
data.insert("binary".into(),v);}
if let Some(mut v)=data.remove("binary_number"){
data.insert("binary_number".into(),v);}
if let Some(mut v)=data.remove("environment"){
if !v.is_null() {
v=decode_Latexmath_Model_Environment(v)?;
}
data.insert("environment".into(),v);}
if let Some(mut v)=data.remove("expression"){
if !v.is_null() {
v=decode_Latexmath_Model_Expression(v)?;
}
data.insert("expression".into(),v);}
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Latexmath_Model_FirstValue(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("fonts"){
data.insert("fonts".into(),v);}
if let Some(mut v)=data.remove("hex_number"){
data.insert("hex_number".into(),v);}
if let Some(mut v)=data.remove("intermediate_exp"){
if !v.is_null() {
v=decode_Latexmath_Model_IntermediateExp(v)?;
}
data.insert("intermediate_exp".into(),v);}
if let Some(mut v)=data.remove("left_right"){
if !v.is_null() {
v=decode_Latexmath_Model_LeftRight(v)?;
}
data.insert("left_right".into(),v);}
if let Some(mut v)=data.remove("limits"){
if !v.is_null() {
v=decode_Latexmath_Model_Limits(v)?;
}
data.insert("limits".into(),v);}
if let Some(mut v)=data.remove("number"){
data.insert("number".into(),v);}
if let Some(mut v)=data.remove("numeric_values"){
data.insert("numeric_values".into(),v);}
if let Some(mut v)=data.remove("octal_number"){
data.insert("octal_number".into(),v);}
if let Some(mut v)=data.remove("operant"){
data.insert("operant".into(),v);}
if let Some(mut v)=data.remove("over"){
if !v.is_null() {
v=decode_Latexmath_Model_Over(v)?;
}
data.insert("over".into(),v);}
if let Some(mut v)=data.remove("power"){
if !v.is_null() {
v=decode_Latexmath_Model_Power(v)?;
}
data.insert("power".into(),v);}
if let Some(mut v)=data.remove("power_base"){
if !v.is_null() {
v=decode_Latexmath_Model_PowerBase(v)?;
}
data.insert("power_base".into(),v);}
if let Some(mut v)=data.remove("row_break"){
data.insert("row_break".into(),v);}
if let Some(mut v)=data.remove("second_value"){
if !v.is_null() {
v=decode_Latexmath_Model_SecondValue(v)?;
}
data.insert("second_value".into(),v);}
if let Some(mut v)=data.remove("space"){
data.insert("space".into(),v);}
if let Some(mut v)=data.remove("subscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Subscript(v)?;
}
data.insert("subscript".into(),v);}
if let Some(mut v)=data.remove("substack"){
data.insert("substack".into(),v);}
if let Some(mut v)=data.remove("supscript"){
if !v.is_null() {
v=decode_Latexmath_Model_Supscript(v)?;
}
data.insert("supscript".into(),v);}
if let Some(mut v)=data.remove("symbol"){
if !v.is_null() {
v=decode_Latexmath_Model_SymbolNode(v)?;
}
data.insert("symbol".into(),v);}
if let Some(mut v)=data.remove("symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("symbols".into(),v);}
if let Some(mut v)=data.remove("table_data"){
if !v.is_null() {
v=decode_Latexmath_Model_TableData(v)?;
}
data.insert("table_data".into(),v);}
if let Some(mut v)=data.remove("ternary"){
data.insert("ternary".into(),v);}
if let Some(mut v)=data.remove("ternary_class"){
if !v.is_null() {
v=decode_Latexmath_Model_TernaryClass(v)?;
}
data.insert("ternary_class".into(),v);}
if let Some(mut v)=data.remove("text"){
data.insert("text".into(),v);}
if let Some(mut v)=data.remove("three_per_em_space"){
data.insert("three_per_em_space".into(),v);}
if let Some(mut v)=data.remove("unary"){
data.insert("unary".into(),v);}
if let Some(mut v)=data.remove("unary_functions"){
if !v.is_null() {
v=decode_Latexmath_Model_UnaryFunctions(v)?;
}
data.insert("unary_functions".into(),v);}
if let Some(mut v)=data.remove("under_over"){
if !v.is_null() {
v=decode_Latexmath_Model_UnderOver(v)?;
}
data.insert("under_over".into(),v);}
if let Some(mut v)=data.remove("underover"){
data.insert("underover".into(),v);}
if let Some(mut v)=data.remove("unicode_symbols"){
if !v.is_null() {
v=decode_Latexmath_Model_MathSymbol(v)?;
}
data.insert("unicode_symbols".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_ThirdValue(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::ThirdValue@default")?;let mut wire=Map::new();
for mut v in data.remove("absent").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("absent".into(),v);}
for mut v in data.remove("base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Base(v)?;
}
wire.insert("base".into(),v);}
for mut v in data.remove("binary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Binary(v)?;
}
wire.insert("binary".into(),v);}
for mut v in data.remove("binary_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("binary_number".into(),v);}
for mut v in data.remove("environment").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Environment(v)?;
}
wire.insert("environment".into(),v);}
for mut v in data.remove("expression").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Expression(v)?;
}
wire.insert("expression".into(),v);}
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_FirstValue(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("fonts").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("fonts".into(),v);}
for mut v in data.remove("hex_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("hex_number".into(),v);}
for mut v in data.remove("intermediate_exp").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_IntermediateExp(v)?;
}
wire.insert("intermediate_exp".into(),v);}
for mut v in data.remove("left_right").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_LeftRight(v)?;
}
wire.insert("left_right".into(),v);}
for mut v in data.remove("limits").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Limits(v)?;
}
wire.insert("limits".into(),v);}
for mut v in data.remove("number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("number".into(),v);}
for mut v in data.remove("numeric_values").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("numeric_values".into(),v);}
for mut v in data.remove("octal_number").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("octal_number".into(),v);}
for mut v in data.remove("operant").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("operant".into(),v);}
for mut v in data.remove("over").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Over(v)?;
}
wire.insert("over".into(),v);}
for mut v in data.remove("power").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Power(v)?;
}
wire.insert("power".into(),v);}
for mut v in data.remove("power_base").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_PowerBase(v)?;
}
wire.insert("power_base".into(),v);}
for mut v in data.remove("row_break").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("row_break".into(),v);}
for mut v in data.remove("second_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SecondValue(v)?;
}
wire.insert("second_value".into(),v);}
for mut v in data.remove("space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("space".into(),v);}
for mut v in data.remove("subscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Subscript(v)?;
}
wire.insert("subscript".into(),v);}
for mut v in data.remove("substack").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("substack".into(),v);}
for mut v in data.remove("supscript").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Supscript(v)?;
}
wire.insert("supscript".into(),v);}
for mut v in data.remove("symbol").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SymbolNode(v)?;
}
wire.insert("symbol".into(),v);}
for mut v in data.remove("symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("symbols".into(),v);}
for mut v in data.remove("table_data").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TableData(v)?;
}
wire.insert("table_data".into(),v);}
for mut v in data.remove("ternary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("ternary".into(),v);}
for mut v in data.remove("ternary_class").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_TernaryClass(v)?;
}
wire.insert("ternary_class".into(),v);}
for mut v in data.remove("text").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("text".into(),v);}
for mut v in data.remove("three_per_em_space").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("three_per_em_space".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("unary".into(),v);}
for mut v in data.remove("unary_functions").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnaryFunctions(v)?;
}
wire.insert("unary_functions".into(),v);}
for mut v in data.remove("under_over").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_UnderOver(v)?;
}
wire.insert("under_over".into(),v);}
for mut v in data.remove("underover").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("underover".into(),v);}
for mut v in data.remove("unicode_symbols").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_MathSymbol(v)?;
}
wire.insert("unicode_symbols".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_UnaryFunctions_json(input:&str)->ContractResult<Latexmath_Model_UnaryFunctions>{let data=decode_Latexmath_Model_UnaryFunctions(parse_json(input,"Latexmath::Model::UnaryFunctions@default")?)?; let model:Latexmath_Model_UnaryFunctions=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::UnaryFunctions@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_UnaryFunctions_json(model:&Latexmath_Model_UnaryFunctions)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::UnaryFunctions@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_UnaryFunctions(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::UnaryFunctions@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_UnaryFunctions(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::UnaryFunctions@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::UnaryFunctions@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("literal") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::UnaryFunctions@default", "literal", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("literal".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("unary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::UnaryFunctions@default", "unary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("unary".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::UnaryFunctions@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_UnaryFunctions(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_UnaryFunctions(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Latexmath_Model_FirstValue(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("literal"){
data.insert("literal".into(),v);}
if let Some(mut v)=data.remove("unary"){
data.insert("unary".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_UnaryFunctions(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::UnaryFunctions@default")?;let mut wire=Map::new();
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_FirstValue(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("literal").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("literal".into(),v);}
for mut v in data.remove("unary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
wire.insert("unary".into(),v);}
Ok(Value::Object(wire))}
#[allow(non_snake_case)]
pub fn read_Latexmath_Model_UnderOver_json(input:&str)->ContractResult<Latexmath_Model_UnderOver>{let data=decode_Latexmath_Model_UnderOver(parse_json(input,"Latexmath::Model::UnderOver@default")?)?; let model:Latexmath_Model_UnderOver=serde_json::from_value(data).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::UnderOver@default", "", "/input",e))?; model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/input",e.reason))?;Ok(model)}
#[allow(non_snake_case)]
pub fn write_Latexmath_Model_UnderOver_json(model:&Latexmath_Model_UnderOver)->ContractResult<String>{model.validate().map_err(|e|failure(ErrorReason::InvalidDocument,e.model_id,e.attribute,"/output",e.reason))?;let data=serde_json::to_value(model).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::UnderOver@default", "", "/output",e))?;serde_json::to_string(&encode_Latexmath_Model_UnderOver(data)?).map_err(|e|failure(ErrorReason::InvalidDocument,"Latexmath::Model::UnderOver@default", "", "/output",e))}
#[allow(non_snake_case)]
fn decode_Latexmath_Model_UnderOver(input:Value)->ContractResult<Value>{let mut wire=object(input,"Latexmath::Model::UnderOver@default")?; let mut data=Map::new();
{ let mut selected=None;
if let Some(v)=wire.remove("binary") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::UnderOver@default", "binary", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("binary".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("first_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::UnderOver@default", "first_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("first_value".into(),v); } }
{ let mut selected=None;
if let Some(v)=wire.remove("second_value") { if selected.is_some() { return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::UnderOver@default", "second_value", "/input", "multiple aliases for member")); } selected=Some(v); }
if let Some(v)=selected { data.insert("second_value".into(),v); } }
if !wire.is_empty(){return Err(failure(ErrorReason::InvalidDocument,"Latexmath::Model::UnderOver@default", "", "/input", "unknown wire key"));} hydrate_Latexmath_Model_UnderOver(data) }
#[allow(non_snake_case)]
fn hydrate_Latexmath_Model_UnderOver(mut data:Map<String,Value>)->ContractResult<Value>{
if let Some(mut v)=data.remove("binary"){
if !v.is_null() {
v=decode_Latexmath_Model_Binary(v)?;
}
data.insert("binary".into(),v);}
if let Some(mut v)=data.remove("first_value"){
if !v.is_null() {
v=decode_Latexmath_Model_FirstValue(v)?;
}
data.insert("first_value".into(),v);}
if let Some(mut v)=data.remove("second_value"){
if !v.is_null() {
v=decode_Latexmath_Model_SecondValue(v)?;
}
data.insert("second_value".into(),v);}
Ok(Value::Object(data))}
#[allow(non_snake_case)]
fn encode_Latexmath_Model_UnderOver(input:Value)->ContractResult<Value>{let mut data=object(input,"Latexmath::Model::UnderOver@default")?;let mut wire=Map::new();
for mut v in data.remove("binary").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_Binary(v)?;
}
wire.insert("binary".into(),v);}
for mut v in data.remove("first_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_FirstValue(v)?;
}
wire.insert("first_value".into(),v);}
for mut v in data.remove("second_value").into_iter(){
if v.is_null(){continue;}
if empty(&v){continue;}
if !v.is_null() {
v=encode_Latexmath_Model_SecondValue(v)?;
}
wire.insert("second_value".into(),v);}
Ok(Value::Object(wire))}
