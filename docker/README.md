# Gestión de Docker

## Flujo Básico

Para ejecutar el contenedor se debe seguir mínimo el siguiente flujo:

1. Crear la imagen
2. Crear el contenedor
3. Ejecutar el contenedor

## Comandos para Imágenes

### Crear Imagen

Para crear la imagen es necesario ejecutar el siguiente comando desde el
directorio `rust/`:

```bash
docker build -t rust-app:latest -f ../docker/Dockerfile .
```

### Eliminar Imagen

```bash
docker rmi rust-app:latest
```

## Comandos para Contenedores

### Crear Contenedor

```bash
docker create --name rust-app rust-app:latest
```

### Ejecutar el Contenedor

```bash
docker start -ai rust-app
```

### Detener el Contenedor

```bash
docker stop rust-app
```

### Crear y Ejecutar el Contenedor

```bash
docker run --name rust-app rust-app:latest
```

> [!TIP]
>
> Se puede añadir la opción `--rm` para eliminar el contenedor automáticamente
> después de detenerlo.

### Eliminar Contenedor

```bash
docker rm rust-app
```

## Comandos Útiles Adicionales

### Ver Logs del Contenedor

```bash
docker logs rust-app
```

### Ejecutar un Comando Dentro del Contenedor

```bash
docker exec rust-app /bin/sh
```

### Abrir una Shell Dentro del Contenedor

```bash
docker exec -it rust-app /bin/sh
```

> [!TIP]
>
> Puedes remplazar `sh` con otra shell disponible (e.g., `bash`, `zsh`).
