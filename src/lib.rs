use serde::{Serialize, Deserialize};
use std::fs;
use wasm_bindgen::prelude::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JvmPath {
    pub path_elements: Vec<String>,
    pub name: String
}

impl JvmPath {
    fn from(path: String) -> Self {
        let elements: Vec<&str> = path.split("/").collect();
        let mut path_elements: Vec<String> = vec![];
        let mut name: &str = "?!";

        let mut i = 0;
        
        for element in &elements {
            if (i+1) == elements.len() {
                name = element
            } else {
                path_elements.push(element.to_string());
            }

            i += 1;
        };

        return Self {
            path_elements: path_elements,
            name: name.to_string()
        }
    }

    fn empty() -> Self {
        return Self {
            path_elements: vec![],
            name: "".to_string()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MappingField {
    pub name: String,
    pub intermediary_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MappingMethod {
    pub jvm_definition: String,
    pub name: String,
    pub intermediary_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MappingsClass {
    pub name: JvmPath,
    pub intermediary_name: JvmPath,
    pub fields: Vec<MappingField>,
    pub methods: Vec<MappingMethod>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Mappings {
    pub classes: Vec<MappingsClass>,
}


fn read_file(file_path: &str) -> String {
    return fs::read_to_string(file_path).expect("couldnt read file");
}

struct MappingsParser {
    pub file_path: String,
    pub current_class: Option<MappingsClass>
}

impl MappingsParser {
    fn parse(&mut self) -> Mappings {
        let mut mappings = Mappings { classes: vec![] };

        let content = read_file(&self.file_path);
        let lines = content.lines();

        lines.for_each(|line| {
            self.parse_line(line, &mut mappings);
        });

        return mappings;
    }

    fn parse_with_content(&mut self, content: &str) -> Mappings {
        let mut mappings = Mappings { classes: vec![] };

        let lines = content.lines();

        lines.for_each(|line| {
            self.parse_line(line, &mut mappings);
        });

        return mappings;
    }

    fn parse_line(&mut self, line: &str, mappings: &mut Mappings) {
        let mut parts = line.trim_matches(|c| c == '\t').split("\t");
        let mut is_class = false;
        let mut is_method = false;
        let mut is_field = false;

        match parts.next() {
            Some(moin) => {
                match moin {
                    "c" => {
                        is_class = true;

                        if self.current_class.is_some() {
                            // BEDENKEN: Evtl klont nicht die vectoren dadrin

                            mappings.classes.push(self.current_class.as_ref().unwrap().clone());
                        }

                        self.current_class = Some(MappingsClass {
                            name: JvmPath::empty(),
                            intermediary_name: JvmPath::empty(),
                            fields: vec![],
                            methods: vec![]
                        })
                    }
                    "m" => {
                        is_method = true;
                    }
                    "f" => {
                        is_field = true;
                    }
                    _ => {
                        println!("unknown part: '{}'", moin);
                        return;
                    }
                }
            }
            None => {
                println!("zeile schon zuend !?");
                return;
            }
        }

        if is_class {
            self.parse_rest_class(&mut parts);
        }

        if is_method {
            self.parse_rest_method(&mut parts);
        }

        if is_field {
            self.parse_rest_field(&mut parts);
        }
    }

    fn parse_rest_class(&mut self, parts: &mut std::str::Split<'_, &str>) {
        if parts.next().is_none() {
            return;
        }

        let intermediary_option = parts.next();
        if intermediary_option.is_none() {
            return;
        }

        let named_option = parts.next();
        if named_option.is_none() {
            return;
        }
        
        self.current_class.as_mut().unwrap().intermediary_name = JvmPath::from(intermediary_option.unwrap().to_string());
        self.current_class.as_mut().unwrap().name = JvmPath::from(named_option.unwrap().to_string());
        
    }

    fn parse_rest_method(&mut self, parts: &mut std::str::Split<'_, &str>) {
        let jvm_function_definition = parts.next();
        if jvm_function_definition.is_none() {
            return;
        }

        if parts.next().is_none() {
            return;
        }
        
        let intermediary_option = parts.next();
        if intermediary_option.is_none() {
            return;
        }
        
        let named_option = parts.next();
        if named_option.is_none() {
            return;
        }

        let function = MappingMethod {
            jvm_definition: jvm_function_definition.unwrap().to_string(),
            name: named_option.unwrap().to_string(),
            intermediary_name: intermediary_option.unwrap().to_string(),
        };
        
        self.current_class.as_mut().unwrap().methods.push(function);
    }

    fn parse_rest_field(&mut self, parts: &mut std::str::Split<'_, &str>) {
        if parts.next().is_none() {
            return;
        }
        

        if parts.next().is_none() {
            return;
        }
        
        let intermediary_option = parts.next();
        if intermediary_option.is_none() {
            return;
        }
        
        let named_option = parts.next();
        if named_option.is_none() {
            return;
        }

        let field = MappingField {
            name: named_option.unwrap().to_string(),
            intermediary_name: intermediary_option.unwrap().to_string(),
        };
        
        self.current_class.as_mut().unwrap().fields.push(field);
    }


    fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
            current_class: Option::None
        }
    }

}


fn file_to_mappings(file_path: &str) -> Mappings {
    let mut parser = MappingsParser::new(file_path);
    let mappings = parser.parse();

    return mappings;
}

trait MappingSafeReplace {
    fn mappings_safe_replace(&self, from: &str, to: &str) -> String;
}

impl MappingSafeReplace for String {
    fn mappings_safe_replace(&self, from: &str, to: &str) -> String {
        let mut result = String::new();
        let mut last_end = 0;

        for (start, part) in self.match_indices(from) {
            let end = start + part.len();
            if end != self.len() {
                let end_char = self.chars().nth(end).unwrap();
                
                match end_char {
                    '0'..='9' => {
                        continue;
                    }
                    _ => {}
                };
            }

            result.push_str(unsafe { self.get_unchecked(last_end..start) });
            result.push_str(to);
            last_end = start + part.len();
        }
        result.push_str(unsafe { self.get_unchecked(last_end..self.len()) });
        
        return result;
    }
}

fn apply_mappings(text: &str, mappings: Mappings) -> String {
    let mut result_text = text.to_string();

    // effizienz busting
    for class in mappings.classes {
        result_text = result_text.mappings_safe_replace(&class.intermediary_name.name, &class.name.name);

        for field in class.fields {
            result_text = result_text.mappings_safe_replace(&field.intermediary_name, &field.name);
        }

        for function in class.methods {
            result_text = result_text.mappings_safe_replace(&function.intermediary_name, &function.name);
        }
    }

    return result_text;
}

fn apply_mappings_from_file(text: &str, mappings_file: &str) -> String {
    let mappings = file_to_mappings(mappings_file);

    return apply_mappings(text, mappings)
}

#[wasm_bindgen]
pub fn apply_mappings_from_mappings_text(text: &str, mappings_text: &str) -> String {
    let mut parser = MappingsParser::new("brain.busitng");
    let mappings = parser.parse_with_content(mappings_text);

    return apply_mappings(text, mappings)
}

#[wasm_bindgen]
pub fn file_to_serialized_mappings(file_path: &str) -> String {
    let mappings = file_to_mappings(file_path);

    let json_str = serde_json::to_string_pretty(&mappings);

    return json_str.unwrap();
}

// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// ich will dass auf github steht dass das ein rust projekt ist 
// zählen kommentare da