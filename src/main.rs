#![allow(dead_code)]
use std::fs;
use std::env;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    // Palabras reservadas
    AsankiKeyword,
    ToyKeyword,
    KametsaKeyword,
    PaiKeyword,
    PaiTeKeyword,
    KamKeyword,
    KaraKeyword,
    TasKeyword,
    PawKeyword,
    IroqKeyword,
    IrokKeyword,
    PawaKeyword,
    ToyaKeyword,
    
    // Valores
    Asanki(String),
    Toy(u32),
    Kametsa(f32),
    
    // Otros
    Variable(String),
    Equal,
    Semicolon,
    Comma,
    LBrace,
    RBrace,
    LParen,
    RParen,
}

struct Lexer {
    texto: Vec<char>,
    pos: usize,
}

impl Lexer {
    fn nuevo(entrada: &str) -> Self {
        Lexer {
            texto: entrada.chars().collect(),
            pos: 0,
        }
    }

    fn obtener_token(&mut self) -> Option<Token> {
        self.saltar_espacios();

        if self.esta_al_final() {
            return None;
        }

        let c = self.texto[self.pos];
        match c {
            '=' => { self.pos += 1; return Some(Token::Equal); }
            ';' => { self.pos += 1; return Some(Token::Semicolon); }
            ',' => { self.pos += 1; return Some(Token::Comma); }
            '{' => { self.pos += 1; return Some(Token::LBrace); }
            '}' => { self.pos += 1; return Some(Token::RBrace); }
            '(' => { self.pos += 1; return Some(Token::LParen); }
            ')' => { self.pos += 1; return Some(Token::RParen); }
            _ => {}
        }

        let inicio = self.pos;
        self.avanzar_mientras_sea_valido();

        if inicio == self.pos {
            self.pos += 1; // saltar carácter no reconocido
            return self.obtener_token();
        }

        let palabra = self.obtener_palabra(inicio);

        self.clasificar_token(palabra)
    }

    fn saltar_espacios(&mut self) {
        while self.pos < self.texto.len() && self.texto[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn esta_al_final(&self) -> bool {
        self.pos >= self.texto.len()
    }

    fn avanzar_mientras_sea_valido(&mut self) {
        while self.pos < self.texto.len() && self.es_caracter_valido(self.texto[self.pos]) {
            self.pos += 1;
        }
    }

    fn es_caracter_valido(&self, c: char) -> bool {
        c.is_alphanumeric() || c == '_' || c == '\'' || c == '.'
    }

    fn obtener_palabra(&self, inicio: usize) -> String {
        self.texto[inicio..self.pos].iter().collect()
    }

    fn clasificar_token(&mut self, palabra: String) -> Option<Token> {
        if self.termina_con_comilla() {
            return Some(Token::Asanki(palabra));
        }

        match palabra.as_str() {
            "toy" => return Some(Token::ToyKeyword),
            "kametsa" => return Some(Token::KametsaKeyword),
            "asanki" => return Some(Token::AsankiKeyword),
            "pai" => return Some(Token::PaiKeyword),
            "pai_te" => return Some(Token::PaiTeKeyword),
            "kam" => return Some(Token::KamKeyword),
            "kara" => return Some(Token::KaraKeyword),
            "tas" => return Some(Token::TasKeyword),
            "paw" => return Some(Token::PawKeyword),
            "iroq" => return Some(Token::IroqKeyword),
            "irok" => return Some(Token::IrokKeyword),
            "pawa" => return Some(Token::PawaKeyword),
            "toya" => return Some(Token::ToyaKeyword),
            _ => {}
        }

        if palabra.contains('.') {
            let valor = palabra.parse::<f32>().expect("Error al parsear flotante");
            Some(Token::Kametsa(valor))
        } else if palabra.chars().all(|c| c.is_ascii_digit()) {
            let valor = palabra.parse::<u32>().expect("Error al parsear entero");
            Some(Token::Toy(valor))
        } else {
            Some(Token::Variable(palabra))
        }
    }

    fn termina_con_comilla(&self) -> bool {
        self.pos > 0 && self.texto[self.pos - 1] == '\''
    }
}

// ==========================================
// ESTRUCTURAS DEL AST
// ==========================================

#[derive(Debug)]
enum Tipo {
    Asanki,
    Toy,
    Kametsa,
}

#[derive(Debug)]
enum Expresion {
    Variable(String),
    Asanki(String),
    Toy(u32),
    Kametsa(f32),
}

#[derive(Debug)]
enum Sentencia {
    // Declaracion: Tipo de dato, nombre de la variable y su valor inicial
    Declaracion(Tipo, String, Expresion),
    // Asignacion: Nombre de variable y su nuevo valor
    Asignacion(String, Expresion),
    // Funcion: Nombre y la lista de sentencias dentro de sus llaves
    Funcion(String, Vec<Sentencia>),
    // Llamada: Imprimir (pawa) una variable
    Llamada(String),
}

#[derive(Debug)]
struct Programa {
    sentencias: Vec<Sentencia>,
}

// ==========================================
// PARSER MINIMO
// ==========================================

struct Parser {
    lexer: Lexer,
    token_actual: Option<Token>,
}

impl Parser {
    fn nuevo(mut lexer: Lexer) -> Self {
        let token_actual = lexer.obtener_token();
        Parser { lexer, token_actual }
    }

    // Avanza al siguiente token generado por el lexer
    fn avanzar(&mut self) {
        self.token_actual = self.lexer.obtener_token();
    }

    // Punto de entrada: parsea todo el texto hasta el final
    fn parsear_programa(&mut self) -> Programa {
        let mut sentencias = Vec::new();
        while self.token_actual.is_some() {
            if let Some(sentencia) = self.parsear_sentencia() {
                sentencias.push(sentencia);
            } else {
                // Si encontramos un token raro, avanzamos para no trabarnos
                self.avanzar();
            }
        }
        Programa { sentencias }
    }

    // Evalúa qué tipo de sentencia estamos leyendo
    fn parsear_sentencia(&mut self) -> Option<Sentencia> {
        let token = self.token_actual.clone()?;

        match token {
            // 1. Declaraciones de variables
            Token::AsankiKeyword | Token::ToyKeyword | Token::KametsaKeyword => {
                let tipo = match token {
                    Token::AsankiKeyword => Tipo::Asanki,
                    Token::ToyKeyword => Tipo::Toy,
                    Token::KametsaKeyword => Tipo::Kametsa,
                    _ => unreachable!(),
                };
                self.avanzar(); // Saltamos el tipo

                // Leemos el nombre
                let nombre = if let Some(Token::Variable(n)) = self.token_actual.clone() {
                    n
                } else { return None; };
                self.avanzar(); // Saltamos el nombre

                // Verificamos el igual '='
                if let Some(Token::Equal) = self.token_actual {
                    self.avanzar(); // Saltamos el '='
                    let valor = self.parsear_expresion()?;
                    
                    // Saltamos el punto y coma ';' opcionalmente
                    if let Some(Token::Semicolon) = self.token_actual { self.avanzar(); }
                    
                    return Some(Sentencia::Declaracion(tipo, nombre, valor));
                }
                None
            }

            // 4. Asignaciones directas a variables
            Token::Variable(nombre) => {
                self.avanzar(); // Saltamos el nombre

                // Verificamos el igual '='
                if let Some(Token::Equal) = self.token_actual {
                    self.avanzar(); // Saltamos el '='
                    let valor = self.parsear_expresion()?;
                    
                    if let Some(Token::Semicolon) = self.token_actual { self.avanzar(); }
                    
                    return Some(Sentencia::Asignacion(nombre, valor));
                }
                None
            }

            // 2. Funciones
            Token::TasKeyword => {
                self.avanzar(); // Saltamos 'tas'
                
                // Obtenemos el nombre de la funcion
                let nombre = if let Some(Token::Variable(n)) = self.token_actual.clone() {
                    n
                } else { return None; };
                self.avanzar(); // Saltamos el nombre

                if let Some(Token::LBrace) = self.token_actual { self.avanzar(); } // Saltamos '{'

                // Parseamos todo lo que está dentro de las llaves
                let mut cuerpo = Vec::new();
                while self.token_actual.is_some() && self.token_actual != Some(Token::RBrace) {
                    if let Some(sentencia) = self.parsear_sentencia() {
                        cuerpo.push(sentencia);
                    } else {
                        self.avanzar();
                    }
                }

                if let Some(Token::RBrace) = self.token_actual { self.avanzar(); } // Saltamos '}'
                
                Some(Sentencia::Funcion(nombre, cuerpo))
            }

            // 3. Llamadas
            Token::PawaKeyword => {
                self.avanzar(); // Saltamos 'pawa'
                
                if let Some(Token::LParen) = self.token_actual { self.avanzar(); } // Saltamos '('
                
                let variable = if let Some(Token::Variable(n)) = self.token_actual.clone() {
                    n
                } else { return None; };
                self.avanzar(); // Saltamos la variable

                if let Some(Token::RParen) = self.token_actual { self.avanzar(); } // Saltamos ')'
                if let Some(Token::Semicolon) = self.token_actual { self.avanzar(); } // Saltamos ';'

                Some(Sentencia::Llamada(variable))
            }

            _ => None,
        }
    }

    // Evalúa los valores a la derecha del '='
    fn parsear_expresion(&mut self) -> Option<Expresion> {
        let token = self.token_actual.clone()?;
        let expresion = match token {
            Token::Variable(n) => Expresion::Variable(n),
            Token::Asanki(s) => Expresion::Asanki(s),
            Token::Toy(v) => Expresion::Toy(v),
            Token::Kametsa(v) => Expresion::Kametsa(v),
            _ => return None,
        };
        self.avanzar();
        Some(expresion)
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let codigo = if args.len() > 1 {
        fs::read_to_string(&args[1]).expect("No se pudo leer el archivo")
    } else {
        "asanki stri = 'Soviet';\ntoy ent = 4;\nkametsa flo = 1.2;\ntoy nose = 5;\n\ntas suma {\n    toy x = 2;\n    pawa(x);\n}".to_string()
    };

    println!("Código a analizar:\n{}", codigo);
    println!("------------------------------------------------");
    println!("AST GENERADO POR EL PARSER MINIMO:\n");

    let lexer = Lexer::nuevo(&codigo);
    let mut parser = Parser::nuevo(lexer);

    let ast = parser.parsear_programa();
    
    // Imprimir el AST resultante en consola
    println!("{:#?}", ast);
}
