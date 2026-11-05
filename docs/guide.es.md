# Guía de Sprixel

[English](guide.md) · **Español**

Esta es la guía larga de Sprixel: cómo instalarlo, cómo usarlo y cómo se conecta
una IA y aprende a usarlo. El [README](../README.es.md) es la versión corta. Para
ver cada detalle, cada parte enlaza a una página de referencia (esas páginas
están en inglés).

## Contenido

1. [Instalar](#instalar)
   - [Método 1: un solo comando](#método-1-un-solo-comando)
   - [Método 2: descargar desde GitHub](#método-2-descargar-desde-github)
   - [La primera vez que lo abres](#la-primera-vez-que-lo-abres)
   - [Actualizar y desinstalar](#actualizar-y-desinstalar)
2. [Usar la app](#usar-la-app)
3. [Conectar una IA](#conectar-una-ia)
4. [Cómo entiende la IA a Sprixel](#cómo-entiende-la-ia-a-sprixel)
   - [Capa 1: las instrucciones del servidor](#capa-1-las-instrucciones-del-servidor)
   - [Capa 2: las herramientas](#capa-2-las-herramientas)
   - [Capa 3: las guías (`get_guide`)](#capa-3-las-guías-get_guide)
   - [Capa 4: la skill de Claude Code](#capa-4-la-skill-de-claude-code)
   - [El ciclo de dibujo](#el-ciclo-de-dibujo)
5. [Tú tienes el control](#tú-tienes-el-control)
6. [Cuando algo sale mal](#cuando-algo-sale-mal)
7. [Dónde leer más](#dónde-leer-más)

---

## Instalar

Sprixel tiene **dos partes**, y necesitas las dos:

| Parte | Qué es |
| --- | --- |
| **La app** | El editor de pixel art (una ventana de escritorio). También ejecuta el servidor MCP con el que habla una IA. |
| **El comando `sprixel`** | Un programa pequeño para la terminal. Conecta tu IA con la app, y puede iniciar, actualizar y revisar Sprixel. |

Los dos métodos de abajo instalan las dos partes. Sistemas compatibles: Linux
x64, macOS (Apple silicon e Intel) y Windows x64. Nada necesita permisos de
administrador.

### Método 1: un solo comando

Es la forma más fácil. El script descarga los archivos de la última versión, los
comprueba y lo pone todo en su sitio.

Linux y macOS:

```sh
curl -fsSL https://github.com/Javier-GarciaP/sprixel/releases/latest/download/install.sh | sh
```

Windows (PowerShell):

```powershell
irm https://github.com/Javier-GarciaP/sprixel/releases/latest/download/install.ps1 | iex
```

Qué hace el script:

- Instala **solo para tu usuario**.
- Descarga los archivos de la versión y **comprueba su SHA-256**. Si una
  comprobación falla, no instala nada.
- Muestra primero el aviso de la licencia MIT.
- Añade el comando `sprixel` a tu PATH si hace falta. **Abre una terminal nueva
  al terminar.**
- Pone Sprixel en el menú de aplicaciones con su icono (en Linux y Windows la
  app escribe la entrada; en macOS la app de `~/Applications` es la entrada).
- Se puede ejecutar otra vez sin problema. Actualiza o repara la instalación y
  conserva tus proyectos, ajustes y tokens.

#### Las preguntas del script

En una terminal, el script muestra un resumen de lo que va a instalar y dónde, y
te pide confirmar (Enter significa sí). Después pregunta:

1. **¿Añadir Sprixel al menú de aplicaciones?** (Linux y Windows.) Un "no" se
   guarda en el archivo `~/.sprixel/no-menu-entry`, así la app no añade la
   entrada por sí sola más tarde.
2. **¿Conectar una herramienta de IA ahora?** Lista solo las que encuentra en tu
   equipo (Claude Code, OpenCode), más "No, lo haré después". Si no encuentra
   ninguna, no pregunta.

Cada descarga muestra su nombre, un contador y una barra de progreso. Si algo
falla (una IA que no se puede conectar, una entrada de menú que no se puede
escribir), recibes un solo aviso que dice cómo reintentar, y la instalación
termina bien.

Para saltar las preguntas (en un script), usa `--yes`. Otras opciones:
`--no-menu` y `--connect claude|opencode|both|none`. Sin terminal, el script
nunca pregunta y usa las respuestas por defecto (instalar, añadir la entrada de
menú, no conectar ninguna IA).

```sh
curl -fsSL https://github.com/Javier-GarciaP/sprixel/releases/latest/download/install.sh | sh -s -- --yes --no-menu --connect claude
```

```powershell
$env:SPRIXEL_YES = "1"; $env:SPRIXEL_CONNECT = "claude"; irm https://github.com/Javier-GarciaP/sprixel/releases/latest/download/install.ps1 | iex
```

Para instalar una versión concreta:

```sh
curl -fsSL https://github.com/Javier-GarciaP/sprixel/releases/latest/download/install.sh | sh -s -- --version 0.1.0
```

```powershell
$env:SPRIXEL_VERSION = "0.1.0"; irm https://github.com/Javier-GarciaP/sprixel/releases/latest/download/install.ps1 | iex
```

Comprueba que funcionó:

```sh
sprixel doctor
```

### Método 2: descargar desde GitHub

Úsalo si no quieres ejecutar un script. Descargas tú mismo los archivos desde la
[página de Releases](https://github.com/Javier-GarciaP/sprixel/releases) y los
pones en su sitio. Necesitas **dos archivos** para tu sistema, más
`checksums.txt` para comprobarlos:

| Sistema | La app | El comando `sprixel` |
| --- | --- | --- |
| Linux | `Sprixel-linux-x64.AppImage` | `sprixel-linux-x64` |
| macOS, Apple silicon | `Sprixel-macos-arm64.app.tar.gz` | `sprixel-macos-arm64` |
| macOS, Intel | `Sprixel-macos-x64.app.tar.gz` | `sprixel-macos-x64` |
| Windows | `Sprixel-windows-x64.zip` | `sprixel-windows-x64.exe` |

**1. Comprueba los archivos.** `checksums.txt` tiene el SHA-256 de cada archivo.
En Linux y macOS, en la carpeta con tus descargas:

```sh
sha256sum -c checksums.txt --ignore-missing       # Linux
shasum -a 256 -c checksums.txt --ignore-missing   # macOS
```

En Windows: `Get-FileHash .\Sprixel-windows-x64.zip` y compara el resultado con
la línea de `checksums.txt`.

**2. Pon los archivos en su sitio.** Son los mismos sitios que usa el script, y
también son los primeros lugares donde `sprixel start` busca la app.

Linux:

```sh
mkdir -p ~/.local/bin
install -m 755 Sprixel-linux-x64.AppImage ~/.local/bin/sprixel-app
install -m 755 sprixel-linux-x64 ~/.local/bin/sprixel
```

macOS (usa `arm64` o `x64` en los nombres):

```sh
mkdir -p ~/Applications ~/.local/bin
tar -xzf Sprixel-macos-arm64.app.tar.gz -C ~/Applications
install -m 755 sprixel-macos-arm64 ~/.local/bin/sprixel
```

Windows (PowerShell):

```powershell
$dir = "$env:LOCALAPPDATA\Sprixel"
New-Item -ItemType Directory -Force "$dir\bin" | Out-Null
Expand-Archive .\Sprixel-windows-x64.zip -DestinationPath $dir -Force
Copy-Item .\sprixel-windows-x64.exe "$dir\bin\sprixel.exe"
```

**3. Añade la carpeta del comando a tu PATH**, para que la terminal encuentre
`sprixel`. Es `~/.local/bin` en Linux y macOS, y `%LOCALAPPDATA%\Sprixel\bin` en
Windows. Abre una terminal nueva y ejecuta `sprixel doctor`.

**4. Pon Sprixel en el menú de aplicaciones** (Linux y Windows). Abre la app una
vez, o ejecútala con `--register-menu`: escribe la entrada y termina sin abrir
ventana.

```sh
~/.local/bin/sprixel-app --register-menu       # Linux
```

```powershell
& "$env:LOCALAPPDATA\Sprixel\Sprixel.exe" --register-menu   # Windows
```

En macOS la app de `~/Applications` ya sale en Launchpad y en Spotlight. Para no
tener entrada, crea el archivo vacío `~/.sprixel/no-menu-entry`.

> La app sola basta para dibujar a mano. Sin el comando `sprixel` no puedes usar
> `sprixel mcp add` para conectar una IA (mira
> [Connect without the command](ai-and-mcp.md#connect-without-the-command)).

### La primera vez que lo abres

Esta versión **no está firmada**, así que tu sistema puede mostrar un aviso.
Es normal.

- **macOS:** haz clic derecho en Sprixel dentro de `~/Applications`, elige
  **Abrir** y confirma una vez. macOS marca los archivos que descargas con el
  navegador; el script de instalación no provoca este aviso.
- **Windows:** en la ventana de SmartScreen elige **Más información** y luego
  **Ejecutar de todas formas**.
- **Linux:** la app es un AppImage. Si no arranca, instala FUSE 2 (`libfuse2` en
  Debian y Ubuntu, `fuse-libs` en Fedora).

### Actualizar y desinstalar

```sh
sprixel update --check      # solo dice si hay una versión nueva
sprixel update              # instala la última versión
sprixel uninstall           # quita Sprixel (tus proyectos se quedan)
```

`sprixel update` comprueba el SHA-256 de cada archivo antes de reemplazar nada.
`sprixel uninstall` pregunta antes de quitar la app, su entrada del menú, el comando,
la entrada del PATH y los registros de la IA. **Nunca borra tus proyectos.**

Más sobre rutas y opciones de instalación: [Usage guide](usage.md#install).

---

## Usar la app

Abre Sprixel desde el menú de aplicaciones, o con `sprixel start`.

1. **Crea un proyecto.** Un proyecto es una carpeta de archivos de texto.
   Funciona bien con Git y se guarda solo.
2. **Crea un sprite.** Elige un tamaño predefinido (de 8x8 a 64x64) o cualquier
   tamaño hasta 512x512.
3. **Dibuja.** Elige un color de la paleta, usa las herramientas y haz zoom con
   Control y la rueda del ratón. Cada trazo se puede deshacer.
4. **Añade frames y animaciones.** Dibuja un frame base y luego crea una
   animación con nombre para cada acción (reposo, caminar, ataque). La vista
   previa la reproduce con su velocidad, y el papel cebolla muestra los frames
   anteriores.
5. **Exporta.** Abre la pestaña Export y escribe una hoja de sprites con un
   archivo JSON, o archivos PNG sueltos, dentro de la carpeta de tu proyecto. La
   exportación a GIF todavía no está lista.

Cada paso tiene más detalle en la [Usage guide](usage.md). Los mismos sprites
también se pueden leer y escribir como texto. Mira el
[text grid format](text-grid-format.md).

---

## Conectar una IA

Una herramienta de IA como **Claude Code** u **OpenCode** puede usar la app
abierta y dibujar en el lienzo real mientras tú miras. Usa el
[Model Context Protocol (MCP)](https://modelcontextprotocol.io/), un estándar
abierto que las herramientas de IA ya conocen.

```
Herramienta de IA ──stdio──▶ sprixel mcp ──HTTP + token, 127.0.0.1──▶ App Sprixel
```

- La **app** es el servidor MCP. Escucha solo en tu propio equipo (`127.0.0.1`),
  nunca en la red.
- El comando **`sprixel mcp`** es un puente pequeño. La herramienta de IA lo
  inicia, y él habla con la app. Si la app está cerrada, la inicia.
- El **token** de acceso es un archivo en `~/.sprixel/`. El puente lo lee por
  ti. Nunca copias un token.

Para conectar tu IA, una sola vez:

```sh
sprixel mcp add claude      # Claude Code
sprixel mcp add opencode    # OpenCode
sprixel doctor              # comprueba que todo funciona
```

O abre la pestaña **Agents** de la app y pulsa **Connect**: te muestra el
comando para copiar y dice **Connected** cuando llega la IA.

La primera vez que una IA se conecta, la app te pide **permitirla** o
**rechazarla** (**Allow** o **Reject**). Hasta que respondas, sus herramientas
fallan con `awaiting_approval`. Después pídele algo a tu IA:

> "Crea un goblin de 16x16 en la carpeta Characters con un ciclo de caminar de
> 4 frames."

Verás un cursor con el nombre de la IA moverse por el lienzo y los píxeles
aparecer en el orden en que se dibujan.

Más: [AI and MCP](ai-and-mcp.md).

---

## Cómo entiende la IA a Sprixel

Una IA no conoce Sprixel de antemano. La app se lo enseña, por capas. Cada capa
va más a fondo, y la IA lee una capa solo cuando la necesita. Así la IA es
rápida y sigue las mismas reglas que un buen artista de pixel art.

### Capa 1: las instrucciones del servidor

Cuando la IA se conecta, el servidor le envía un texto corto con el **flujo de
trabajo** y un ejemplo resuelto (un slime de 16x16 con un frame de reposo y uno
de salto). La IA lo recibe con cualquier cliente MCP, sin instalar nada más. El
flujo es:

1. `get_project`: lee el proyecto.
2. `get_guide`: lee la guía para el trabajo (mira la capa 3).
3. `get_palette`: lee los colores, y dibuja solo con esas teclas.
4. `create_sprite` o `get_sprite`: crea o lee un sprite.
5. Dibuja con `apply_grid`, `set_pixels` o `draw_shape`.
6. **Mira la imagen** que devuelve cada escritura, corrige lo que esté mal y
   vuelve a mirar.

### Capa 2: las herramientas

La IA ve más de 45 herramientas, y cada una tiene una descripción que ya dice
cómo usarla. Están agrupadas por lo que hacen, y cada grupo tiene su permiso:

| Grupo | Ejemplos | Permiso |
| --- | --- | --- |
| Leer | `get_project`, `get_sprite`, `get_palette`, `render_preview`, `analyze_sprite` | Siempre permitido |
| Crear | `create_sprite`, `create_folder`, `add_frame` | Create sprites |
| Editar | `apply_grid`, `set_pixels`, `draw_shape`, `auto_outline`, `auto_shade`, `transform_region`, `create_animation`, `set_palette_color` | Edit pixels |
| Exportar | `export` | Export files |
| Borrar | `delete_sprite` | Delete sprites (**desactivado** por defecto) |
| Abrir proyectos | `open_project` | Open projects (**desactivado** por defecto) |

La IA dibuja con una **cuadrícula de texto**: un carácter por píxel, una línea
por fila, donde `.` siempre es transparente. Esto es lo que recibe `apply_grid`:

```
legend
  K outline #1E3A2B
  B base #6FD36B
frame 1
.KKKK.
KBBBBK
.KKKK.
```

Cada error tiene un `code` estable y un `message` que dice cómo arreglarlo (por
ejemplo `row 7 has 7 columns, expected 8`), así la IA puede corregirse sola. La
lista completa de herramientas está en
[AI and MCP](ai-and-mcp.md#what-the-ai-can-do-tools).

### Capa 3: las guías (`get_guide`)

`get_guide` es una herramienta que devuelve **consejos de pixel art**, según el
`topic` (tema). La IA pide la guía que encaja con el trabajo. Las guías viven
dentro de la app, así que siempre coinciden con las herramientas de esa versión.
Hay nueve:

| Tema | Qué enseña |
| --- | --- |
| `design` | El método: una paleta pequeña con rampas, una sola dirección de luz, primero la silueta, rellenos planos, dos o tres tonos de sombra, un contorno selectivo, sin píxeles sueltos |
| `animation` | Cómo animar: primero las poses clave, la misma paleta en todos los frames, un bucle que cierra |
| `animations-from-base` | Dibuja un frame base, crea una animación con nombre por acción y cambia solo lo que se mueve |
| `palette` | Cómo elegir y construir los colores |
| `large-sprites` | Sprites de más de 32x32: trabajar por regiones con `x` e `y` |
| `faces-and-animals` | Cómo dibujar caras y animales |
| `walk-and-attack-cycles` | Cómo hacer animaciones de caminar y de ataque |
| `game-ready` | Lo que un juego necesita de un sprite: un solo tamaño, anclas (pies, mano, cabeza), tiempos de frame, tiles sin costuras y la exportación |
| `tools` | Un mapa de qué herramienta usar para qué |

La mayoría de las guías tienen un ejemplo resuelto con las llamadas exactas, y
una prueba ejecuta esas llamadas con las herramientas reales, así los ejemplos
no se quedan viejos.

Dos reglas se repiten en todas partes (las instrucciones del servidor, las guías
y la skill):

- **Primero la base, después las animaciones.** Usa `create_animation`, que
  copia el frame base, y cambia solo lo que se mueve.
- **Nunca hagas píxeles fuera de la app** (scripts, librerías de imágenes) salvo
  que tú estés de acuerdo.

### Capa 4: la skill de Claude Code

`sprixel mcp add claude` también escribe una skill en
`~/.claude/skills/sprixel/SKILL.md`. Lleva el mismo flujo, el mismo ejemplo y
las reglas de diseño, y añade cómo conectarse y qué hacer cuando no hay
herramientas MCP (solo una shell, usando `sprixel tool ...`). La skill viene
dentro del comando `sprixel`, así que no se descarga nada. Una prueba falla si
la skill y el servidor dicen cosas distintas. `sprixel mcp remove claude` y
`sprixel uninstall` borran su carpeta y nada más de Claude Code.

La skill es solo para Claude Code. OpenCode y otros clientes aprenden de las
capas 1 a 3, que de todas formas son la fuente principal.

### El ciclo de dibujo

Una IA trabaja mejor cuando puede **ver** su trabajo. Cada herramienta que
cambia píxeles devuelve un PNG del frame que editó. La IA debe mirarlo y
corregir lo que esté mal antes de decir que terminó. Otras herramientas la
ayudan a ir seguro:

- `dry_run` en las ediciones grandes muestra lo que **pasaría** (conteos,
  avisos y una imagen) y no escribe nada.
- `analyze_sprite` encuentra píxeles sueltos, huecos en el contorno y colores
  sin usar, y dice qué no revisó.
- `render_frames` y `render_onion` muestran varios frames a la vez.
- `base_revision` hace que una escritura falle con `stale_revision` si el
  proyecto cambió, así la IA vuelve a leer en vez de pisar tu trabajo.

---

## Tú tienes el control

- **Tú apruebas** cada cliente de IA nuevo, y una versión nueva de un cliente
  vuelve a preguntar.
- **Tú eliges** qué puede hacer con los interruptores de permisos de la pestaña
  **Agents**. Un interruptor se aplica a la siguiente llamada, sin reconectar.
- **Puedes detenerla.** **Stop** cancela la llamada en curso, y las siguientes
  escrituras de la IA fallan con `stopped_by_user`.
- **Puedes deshacerla.** **Undo** revierte la última acción de la IA, como un
  solo paso.
- **Ves todo** lo que hizo en la lista de actividad, con su nombre y la hora.
- **Está cerrado por defecto.** El servidor escucha solo en `127.0.0.1`, pide un
  token aleatorio en cada petición, rechaza las peticiones de páginas web y solo
  escribe exportaciones dentro de la carpeta de tu proyecto.

Detalles y límites: [AI and MCP: Permissions](ai-and-mcp.md#permissions) y
[Security](ai-and-mcp.md#security).

---

## Cuando algo sale mal

Ejecuta primero `sprixel doctor`. Revisa la instalación, si la app está abierta,
el token y los registros de tu IA, y dice cómo arreglar lo que falla.

| Problema | Qué probar |
| --- | --- |
| La IA no encuentra Sprixel | Ejecuta `sprixel mcp add claude` otra vez, y luego `sprixel doctor` |
| Las herramientas fallan con `awaiting_approval` | Pulsa **Allow** para la IA en la pestaña Agents |
| La app está cerrada | Ejecuta `sprixel start` |
| Una acción de la IA salió mal | Pulsa **Undo** en el editor o en la pestaña Agents |
| La app no arranca en Linux | Instala FUSE 2 (`libfuse2` o `fuse-libs`) |

Más: [Usage guide: Troubleshooting](usage.md#troubleshooting).

---

## Dónde leer más

Estas páginas están en inglés.

| Página | Qué contiene |
| --- | --- |
| [Usage guide](usage.md) | Cada pantalla y atajo de la app, y el comando `sprixel` |
| [AI and MCP](ai-and-mcp.md) | Cada herramienta, cada código de error, permisos y seguridad |
| [Text grid format](text-grid-format.md) | El formato de texto que usa una IA para dibujar |
| [Export format](export-format.md) | La hoja de sprites, el JSON y los archivos PNG |
| [Game engines](game-engines.md) | De un dibujo a Godot 4 y Unity |
