# Sprixel installer for Windows (PowerShell 5.1 or newer).
#
#   irm https://github.com/Javier-GarciaP/sprixel/releases/latest/download/install.ps1 | iex
#   $env:SPRIXEL_VERSION = "0.2.0"; irm .../install.ps1 | iex     (a specific version)
#   $env:SPRIXEL_YES = "1"; $env:SPRIXEL_MENU = "0"; $env:SPRIXEL_CONNECT = "claude"; irm .../install.ps1 | iex
#
# It installs for the current user only (no administrator rights). It downloads
# the files of a GitHub Release, checks their SHA-256 and only then copies them:
#   %LOCALAPPDATA%\Sprixel\Sprixel.exe       the app
#   %LOCALAPPDATA%\Sprixel\bin\sprixel.exe   the command (added to your PATH)
# Your projects and the folder %USERPROFILE%\.sprixel (settings, tokens) are
# never touched, so running it again upgrades or repairs the installation.
#
# On a terminal it shows a summary, asks you to confirm and asks two questions
# (the Start Menu shortcut, and connecting an AI). With no terminal, or with
# SPRIXEL_YES=1, it asks nothing and uses the default answers.
#
# Variables: SPRIXEL_VERSION, SPRIXEL_YES=1, SPRIXEL_MENU=0, and
# SPRIXEL_CONNECT=claude|opencode|both|none.
# For tests only: SPRIXEL_BASE_URL replaces the download folder (for example
# file:///C:/release), SPRIXEL_ARCH replaces the CPU type, SPRIXEL_SKIP_PATH=1
# leaves the PATH of the user alone, and SPRIXEL_TTY is "none" (no terminal) or
# the path of a file with the answers.

function Install-Sprixel {
    $ErrorActionPreference = "Stop"
    $repo = "Javier-GarciaP/sprixel"
    $source = "https://github.com/$repo"
    $version = "$env:SPRIXEL_VERSION".TrimStart("v")
    $yes = [bool]$env:SPRIXEL_YES
    $menu = "$env:SPRIXEL_MENU"
    $connect = "$env:SPRIXEL_CONNECT"
    if ($connect -notin @("", "claude", "opencode", "both", "none")) {
        throw "SPRIXEL_CONNECT must be claude, opencode, both or none (not `"$connect`")."
    }

    Write-Host "Sprixel installer"
    Write-Host "Sprixel is open source software released under the MIT License."
    Write-Host "Source code: $source"
    Write-Host "The license and the third-party notices are installed in %LOCALAPPDATA%\Sprixel\"
    Write-Host ""

    # 1. Which system is this? Only Windows on x64 has files in the release.
    $arch = if ($env:SPRIXEL_ARCH) { $env:SPRIXEL_ARCH } else { $env:PROCESSOR_ARCHITECTURE }
    if ($arch -notin @("AMD64", "x86_64", "x64")) {
        throw "this system is not supported by install.ps1 (CPU: $arch). Supported: Windows x64. For Linux and macOS use install.sh."
    }
    if (-not $env:LOCALAPPDATA) { throw "LOCALAPPDATA is not set. Is this Windows?" }
    $userHome = if ($env:USERPROFILE) { $env:USERPROFILE } else { $HOME }

    # 2. Where do the files come from?
    if ($env:SPRIXEL_BASE_URL) { $base = $env:SPRIXEL_BASE_URL }
    elseif ($version) { $base = "$source/releases/download/v$version" }
    else { $base = "$source/releases/latest/download" }

    $cli = "sprixel-windows-x64.exe"
    $app = "Sprixel-windows-x64.zip"

    # 3. Is there a terminal to ask on? Run with `irm | iex` the script comes
    # through the pipeline, and Read-Host still reads the keyboard.
    $tty = @{ file = $false; lines = @(); index = 0 }
    $interactive = $false
    if (-not $yes) {
        if ($env:SPRIXEL_TTY -eq "none") { $interactive = $false }
        elseif ($env:SPRIXEL_TTY) {
            $tty.file = $true
            $tty.lines = @(Get-Content -LiteralPath $env:SPRIXEL_TTY)
            $interactive = $true
        }
        elseif ([Environment]::UserInteractive -and -not [Console]::IsInputRedirected) { $interactive = $true }
    }

    function Read-Answer($prompt) {
        if ($tty.file) {
            Write-Host -NoNewline "$prompt "
            $line = ""
            if ($tty.index -lt $tty.lines.Count) { $line = "$($tty.lines[$tty.index])"; $tty.index++ }
            Write-Host ""
            return $line
        }
        return (Read-Host $prompt)
    }

    # Enter, or no more lines, gives the default.
    function Ask-Yes($prompt, $default) {
        while ($true) {
            $answer = (Read-Answer $prompt).Trim()
            if ($answer -eq "") { $answer = $default }
            if ($answer -match '^(y|yes)$') { return $true }
            if ($answer -match '^(n|no)$') { return $false }
            Write-Host "Please answer y or n."
        }
    }

    # 4. The summary: what will be installed, and where.
    Write-Host "This will install:"
    Write-Host ("  Version   " + $(if ($version) { $version } else { "the latest" }))
    Write-Host "  System    windows x64"
    Write-Host "  App       %LOCALAPPDATA%\Sprixel\Sprixel.exe"
    Write-Host "  Command   %LOCALAPPDATA%\Sprixel\bin\sprixel.exe"
    Write-Host "  Notices   %LOCALAPPDATA%\Sprixel\"
    Write-Host ""

    if ($interactive) {
        if (-not (Ask-Yes "Install Sprixel? [Y/n]" "y")) {
            Write-Host "Nothing was installed."
            return
        }
        Write-Host ""
    }

    # 5. The options. A variable answers the question before it is asked.
    if (-not $menu) {
        $menu = "1"
        if ($interactive -and -not (Ask-Yes "Add Sprixel to your Start Menu? [Y/n]" "y")) { $menu = "0" }
    }

    # The AI tools that this computer has. With none, the question is not asked.
    if ($interactive -and -not $connect) {
        $hasClaude = [bool](Get-Command claude -ErrorAction SilentlyContinue)
        $hasOpenCode = [bool](Get-Command opencode -ErrorAction SilentlyContinue)
        if ($hasClaude -or $hasOpenCode) {
            $choices = @("none")
            if ($hasClaude) { $choices += "claude" }
            if ($hasOpenCode) { $choices += "opencode" }
            if ($hasClaude -and $hasOpenCode) { $choices += "both" }
            $labels = @{ none = "No, I will do it later"; claude = "Claude Code"; opencode = "OpenCode"; both = "Both" }
            Write-Host "Connect an AI tool to Sprixel now?"
            for ($i = 0; $i -lt $choices.Count; $i++) { Write-Host ("  " + ($i + 1) + ") " + $labels[$choices[$i]]) }
            while ($true) {
                $answer = (Read-Answer ("Choose 1-" + $choices.Count + " [1]")).Trim()
                if ($answer -eq "") { $answer = "1" }
                if ($answer -match '^\d{1,2}$' -and [int]$answer -ge 1 -and [int]$answer -le $choices.Count) {
                    $connect = $choices[[int]$answer - 1]
                    break
                }
                Write-Host ("Please type a number from 1 to " + $choices.Count + ".")
            }
            Write-Host ""
        }
    }
    if (-not $connect) { $connect = "none" }

    $tmp = Join-Path ([IO.Path]::GetTempPath()) ("sprixel-" + [Guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $tmp | Out-Null

    try {
        # 6. Download everything and check it BEFORE installing anything. On a
        # terminal curl.exe draws a progress bar; otherwise each file is one
        # plain line.
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        $total = 6
        $progress = @{ n = 0 }
        $useBar = -not [Console]::IsOutputRedirected
        $curl = Get-Command curl.exe -ErrorAction SilentlyContinue
        function Get-Release($name) {
            $target = Join-Path $tmp $name
            $progress.n++
            $label = "[" + $progress.n + "/$total] $name"
            if ($useBar) { Write-Host $label } else { Write-Host -NoNewline "$label ... " }
            try {
                if ($base.StartsWith("file://")) {
                    Copy-Item -LiteralPath ([Uri]"$base/$name").LocalPath -Destination $target
                } elseif ($curl) {
                    $mode = if ($useBar) { "--progress-bar" } else { "-sS" }
                    & $curl.Source --proto "=https" --proto-redir "=https" -fL $mode "$base/$name" -o $target
                    if ($LASTEXITCODE -ne 0) { throw "curl.exe failed" }
                } else {
                    # The default progress bar makes Invoke-WebRequest very slow in PowerShell 5.1.
                    $ProgressPreference = "SilentlyContinue"
                    Invoke-WebRequest -UseBasicParsing -Uri "$base/$name" -OutFile $target
                }
            } catch {
                if (-not $useBar) { Write-Host "" }
                throw "cannot download $base/$name (is the version right, and are you online?)"
            }
            if (-not $useBar) { Write-Host "done" }
        }

        Write-Host "Downloading Sprixel (windows x64)..."
        Get-Release "checksums.txt"
        $sums = @{}
        foreach ($line in Get-Content (Join-Path $tmp "checksums.txt")) {
            $parts = $line -split "\s+", 2
            if ($parts.Count -eq 2) { $sums[$parts[1].Trim()] = $parts[0].ToLower() }
        }
        $names = @("VERSION", $cli, $app, "LICENSE", "THIRD-PARTY.md")
        foreach ($name in $names) { Get-Release $name }
        foreach ($name in $names) {
            if (-not $sums.ContainsKey($name)) { throw "checksums.txt has no checksum for $name. Nothing was installed." }
            $got = (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $tmp $name)).Hash.ToLower()
            if ($got -ne $sums[$name]) { throw "the checksum of $name does not match. The download is damaged or was changed. Nothing was installed." }
        }
        Write-Host "Checksums are correct."

        # 4. Install into the user folder.
        $home_ = Join-Path $env:LOCALAPPDATA "Sprixel"
        $bin = Join-Path $home_ "bin"
        New-Item -ItemType Directory -Force -Path $bin | Out-Null

        # A running program cannot be overwritten on Windows, but it can be
        # renamed: move the old file away, then copy the new one.
        function Put-File($from, $to) {
            if (Test-Path -LiteralPath $to) {
                Remove-Item -LiteralPath "$to.old" -Force -ErrorAction SilentlyContinue
                Move-Item -LiteralPath $to -Destination "$to.old" -Force
            }
            Copy-Item -LiteralPath $from -Destination $to -Force
            Remove-Item -LiteralPath "$to.old" -Force -ErrorAction SilentlyContinue
        }

        $unpacked = Join-Path $tmp "app"
        Expand-Archive -LiteralPath (Join-Path $tmp $app) -DestinationPath $unpacked
        if (-not (Test-Path -LiteralPath (Join-Path $unpacked "Sprixel.exe"))) {
            throw "the app archive has no Sprixel.exe. Nothing was installed."
        }
        Put-File (Join-Path $tmp $cli) (Join-Path $bin "sprixel.exe")
        Put-File (Join-Path $unpacked "Sprixel.exe") (Join-Path $home_ "Sprixel.exe")
        Copy-Item -LiteralPath (Join-Path $tmp "LICENSE") -Destination (Join-Path $home_ "LICENSE") -Force
        Copy-Item -LiteralPath (Join-Path $tmp "THIRD-PARTY.md") -Destination (Join-Path $home_ "THIRD-PARTY.md") -Force
        $installed = (Get-Content (Join-Path $tmp "VERSION") -Raw).Trim()

        # 5. Make sure `sprixel` is on the PATH of new terminals.
        $newTerminal = $false
        if ($env:SPRIXEL_SKIP_PATH -ne "1") {
            $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
            $entries = @($userPath -split ";" | Where-Object { $_ })
            if ($entries -notcontains $bin) {
                [Environment]::SetEnvironmentVariable("Path", (($entries + $bin) -join ";"), "User")
                $newTerminal = $true
                Write-Host "Added $bin to your PATH."
            }
        }

        # 6. The Start Menu. A "no" is kept in %USERPROFILE%\.sprixel\no-menu-entry,
        # which the app reads, so it does not add the shortcut by itself later.
        # A "yes" removes an old "no", and asks the app to put itself in the
        # Start Menu (a shortcut with its icon). A problem here is a warning,
        # never an error: the app does the same the first time it opens. The
        # app has no console window, so it is started with Start-Process and
        # waited for, with a time limit.
        $noMenuFile = Join-Path (Join-Path $userHome ".sprixel") "no-menu-entry"
        Write-Host ""
        if ($menu -eq "0") {
            New-Item -ItemType Directory -Force -Path (Split-Path $noMenuFile) | Out-Null
            New-Item -ItemType File -Force -Path $noMenuFile | Out-Null
            Write-Host "Skipped the Start Menu shortcut (you can add it later with: Sprixel.exe --register-menu)."
            $menuResult = "no"
        } else {
            Remove-Item -LiteralPath $noMenuFile -Force -ErrorAction SilentlyContinue
            Write-Host "Adding Sprixel to the Start Menu..."
            $menuOk = $false
            try {
                $registered = Start-Process -FilePath (Join-Path $home_ "Sprixel.exe") -ArgumentList "--register-menu" -PassThru -WindowStyle Hidden
                if ($registered.WaitForExit(30000)) { $menuOk = ($registered.ExitCode -eq 0) }
                else { $registered.Kill() }
            } catch {
                $menuOk = $false
            }
            if ($menuOk) {
                Write-Host "Sprixel is in the Start Menu."
                $menuResult = "yes"
            } else {
                Write-Host "Sprixel could not be added to the Start Menu. It will try again the first"
                Write-Host "time you open the app, or you can run: Sprixel.exe --register-menu"
                $menuResult = "not done"
            }
        }

        # 7. Connect the AI tools that were chosen. The command is run by its
        # full path, because the PATH of this terminal has not changed yet.
        $result = @{ connected = @() }
        function Connect-One($client, $name) {
            Write-Host ""
            Write-Host "Connecting $name..."
            $ok = $false
            try {
                & (Join-Path $bin "sprixel.exe") mcp add $client
                $ok = ($LASTEXITCODE -eq 0)
            } catch {
                $ok = $false
            }
            if ($ok) { $result.connected += $name }
            else { Write-Host "warning: could not connect $name. You can try again later with: sprixel mcp add $client" }
        }
        if ($connect -eq "claude" -or $connect -eq "both") { Connect-One "claude" "Claude Code" }
        if ($connect -eq "opencode" -or $connect -eq "both") { Connect-One "opencode" "OpenCode" }

        # 8. The summary and the next steps.
        Write-Host ""
        Write-Host "Sprixel $installed is installed."
        Write-Host ""
        Write-Host "  Command   %LOCALAPPDATA%\Sprixel\bin\sprixel.exe"
        Write-Host "  App       %LOCALAPPDATA%\Sprixel\Sprixel.exe"
        Write-Host "  Choices   menu entry: $menuResult"
        Write-Host ("  AI tools: " + $(if ($result.connected.Count -gt 0) { $result.connected -join ", " } else { "none connected" }))
        if ($newTerminal) {
            Write-Host ""
            Write-Host "Open a new terminal so that the 'sprixel' command is found."
        }
        Write-Host ""
        Write-Host "Next steps:"
        Write-Host "  sprixel doctor            check that everything works"
        if ($result.connected.Count -eq 0) {
            Write-Host "  sprixel mcp add claude    let Claude Code use Sprixel (or: opencode)"
        }
        Write-Host ""
        Write-Host "The app is not signed yet. If Windows SmartScreen warns you the first"
        Write-Host "time, choose 'More info' and then 'Run anyway'."
    }
    finally {
        Remove-Item -LiteralPath $tmp -Recurse -Force -ErrorAction SilentlyContinue
    }
}

try {
    Install-Sprixel
} catch {
    Write-Host "error: $($_.Exception.Message)" -ForegroundColor Red
    # Run as a file (tests) it sets the exit code. Run with `| iex` it must not
    # close the terminal of the user, so it only prints the error.
    if ($MyInvocation.MyCommand.Path) { exit 1 }
}
