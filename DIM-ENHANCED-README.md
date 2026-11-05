# Dim Enhanced — build para ZimaOS

Este fork agrega un rediseño visual inspirado en Netflix a la interfaz web de Dim
(proyecto original: [Dusk-Labs/dim](https://github.com/Dusk-Labs/dim)). El cambio es
de presentación: conserva las rutas, la API, el escaneo de biblioteca, la
autenticación y el reproductor originales.

Repo: **https://github.com/marcodido69/dim-enhanced**
Imagen: **ghcr.io/marcodido69/dim-enhanced:latest**

## Qué cambió en este fork

- `ui/src/Enhanced.scss`: tema oscuro/cinematográfico, acento rojo, banner grande,
  carruseles horizontales de posters y layout responsive.
- El tema se carga desde `ui/src/App.jsx`.
- `.github/workflows/dim-enhanced.yml`: construye y publica la imagen Docker en
  GitHub Container Registry (GHCR) en cada push a `main`.
- `docker-compose-zimaos.yml`: compose listo para el equipo ZimaOS.
- Los workflows heredados del upstream que no aplican (Rust, React, docs) quedan
  solo con ejecución manual, para que no fallen ni consuman minutos en cada push.

## Cómo se publica la imagen

1. Hacé push a `main` en https://github.com/marcodido69/dim-enhanced (o corré el
   workflow **Build and publish Dim Enhanced** a mano desde la pestaña Actions).
2. Esperá a que el job `docker` termine. Tarda ~30–60 minutos la primera vez,
   porque compila la UI y el backend Rust dentro de la imagen; las siguientes
   corridas reutilizan la cache de Docker.
3. La imagen generada es `ghcr.io/marcodido69/dim-enhanced:latest`.
4. El workflow intenta dejar el paquete público automáticamente. Si no lo logra,
   entrá al paquete en GitHub → **Package settings** → **Change visibility** →
   **Public**. Es necesario para que ZimaOS pueda hacer `docker pull` sin login.

Notas:

- La imagen es **linux/amd64** (ZimaOS corre en x86). Si algún día necesitás arm64,
  agregá `linux/amd64,linux/arm64` en `platforms:` del workflow, sabiendo que el
  build tarda mucho más por la emulación.
- No se publican binarios sueltos: solo la imagen.

## Desplegar en ZimaOS

1. Copiá `docker-compose-zimaos.yml` al equipo (por ejemplo a
   `/DATA/AppData/dim-enhanced/docker-compose.yml`).
2. Revisá las rutas montadas:
   - Config: `/DATA/AppData/dim-enhanced/config` → `/opt/dim/config`
   - Películas: `/media/1TBWD_1/Media/Movies` → `/media/movies` (solo lectura)
   - Web: puerto `8000`
3. Si tu ruta de medios es otra, cambiala en el compose.
4. Levantá el contenedor:
   ```bash
   docker compose -f docker-compose-zimaos.yml up -d
   ```
5. Entrá a `http://IP-DEL-ZIMAOS:8000`, creá la biblioteca de películas y usá
   `/media/movies` como ruta dentro del contenedor.

Si ZimaOS no tiene `/dev/dri` o la aceleración falla, quitá el bloque `devices:` y
volvé a levantar el contenedor.

## Importante

- El `Dockerfile` original y sus dependencias se mantienen intactos.
- Conservá tu contenedor y configuración actuales de Dim hasta probar esta imagen.
- El proyecto está bajo licencia AGPL-3.0 (`LICENSE.md`). Si distribuís la imagen,
  el código fuente de estas modificaciones queda disponible en este mismo repo.
