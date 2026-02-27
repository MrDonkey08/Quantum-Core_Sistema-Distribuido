# Políticas y Procedimientos de Seguridad

Este documento describe los procedimientos de seguridad y las políticas
generales para el proyecto de sistema distribuido de Mandelbrot.

## Tabla de Contenidos

<!-- toc -->

- [Buenas Prácticas de Seguridad](#buenas-prácticas-de-seguridad)
  - [Protección de Datos Sensibles](#protección-de-datos-sensibles)
  - [Configuración de `.gitignore`](#configuración-de-.gitignore)
    - [Recursos Recomendados](#recursos-recomendados)
- [Comentarios sobre esta Política](#comentarios-sobre-esta-política)

<!-- tocstop -->

## Buenas Prácticas de Seguridad

### Protección de Datos Sensibles

Para mantener la tríada CID (Confidencialidad, Integridad, Disponibilidad):
**Nunca hagas commit que contengan:**

- Llaves privadas (e.g., SSH, TLS, GPG)
- Tokens de API o credenciales
- Archivos de entorno con secretos (`.env` con passwords, tokens)
- Cualquier otro dato sensible

**Si accidentalmente haces commits con datos sensibles:** sigue la guía de
GitHub:
[Removing sensitive data from a repository](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/removing-sensitive-data-from-a-repository)

### Configuración de `.gitignore`

El repositorio incluye un `.gitignore` con reglas de seguridad. Antes de hacer
commit:

1. Revisa la sección de seguridad en `.gitignore`
2. Verifica que no hay secretos staged: `git diff --cached`
3. Usa `git-secrets` o `gitleaks` para escaneo automatizado (opcional pero
   recomendado)

#### Recursos Recomendados

En caso de actualizar el `.gitignore` considera revisar los siguientes recursos:

- [Documentación oficial de gitignore](https://git-scm.com/docs/gitignore)
- [gitignore.io templates](https://www.toptal.com/developers/gitignore)
- [GitHub gitignore templates](https://github.com/github/gitignore)

## Comentarios sobre esta Política

Si tienes sugerencias sobre cómo mejorar este proceso, por favor, abre un pull
request.
