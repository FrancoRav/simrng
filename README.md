# SimRNG

Backend para el TP2 de la cátedra de Simulación, 2023

- Escrito en Rust 🦀
- Librería y API web

## Setup

Se necesita tener Rust instalado. [Instrucciones](https://www.rust-lang.org/tools/install)

### Compilar y ejecutar para desarrollo
```sh
cargo run
```

### Compilar con optimizaciones y ejecutar
```sh
cargo run --release
```

### Compilar con optimizaciones
```sh
cargo build --release
```
El ejecutable se encontrará en `target/release/simrng`

## Uso

- Ejecutar el proyecto, se iniciará el servidor en el puerto 3000
- Abrir el servidor de frontend (con `npm run dev` o algún servidor web, nginx/apache/otros), por defecto en el puerto 5173
- Entrar a http://127.0.0.1:5173/ en el navegador
