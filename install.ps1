# Sprixel installer for Windows (PowerShell 5.1 or newer).
#
#   irm https://github.com/Javier-GarciaP/sprixel/releases/latest/download/install.ps1 | iex
#   $env:SPRIXEL_VERSION = "0.2.0"; irm .../install.ps1 | iex     (a specific version)
#
# It installs for the current user only (no administrator rights). It downloads
# the files of a GitHub Release, checks their SHA-256 and only then copies them:
#   %LOCALAPPDATA%\Sprixel\Sprixel.exe       the app
#   %LOCALAPPDATA%\Sprixel\bin\sprixel.exe   the command (added to your PATH)
# Your projects and the folder %USERPROFILE%\.sprixel (settings, tokens) are
# never touched, so running it again upgrades or repairs the installation.
#
# SPRIXEL_BASE_URL (tests only) replaces the download folder, for example
# file:///C:/release. SPRIXEL_ARCH (tests only) replaces the CPU type.
# SPRIXEL_SKIP_PATH=1 (tests only) leaves the PATH of the user alone.

function Install-Sprixel {
    $ErrorActionPreference = "Stop"
    $repo = "Javier-GarciaP/sprixel"
    $source = "https://github.com/$repo"
    $version = "$env:SPRIXEL_VERSION".TrimStart("v")

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

    # 2. Where do the files come from?
    if ($env:SPRIXEL_BASE_URL) { $base = $env:SPRIXEL_BASE_URL }
    elseif ($version) { $base = "$source/releases/download/v$version" }
    else { $base = "$source/releases/latest/download" }

    $cli = "sprixel-windows-x64.exe"
    $app = "Sprixel-windows-x64.zip"
    $tmp = Join-Path ([IO.Path]::GetTempPath()) ("sprixel-" + [Guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $tmp | Out-Null

    try {
        # 3. Download everything and check it BEFORE installing anything.
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        function Get-Release($name) {
            $target = Join-Path $tmp $name
            try {
                if ($base.StartsWith("file://")) {
                    Copy-Item -LiteralPath ([Uri]"$base/$name").LocalPath -Destination $target
                } else {
                    Invoke-WebRequest -UseBasicParsing -Uri "$base/$name" -OutFile $target
                }
            } catch {
                throw "cannot download $base/$name (is the version right, and are you online?)"
            }
        }

        Write-Host "Downloading Sprixel (windows x64)..."
        Get-Release "checksums.txt"
        $sums = @{}
        foreach ($line in Get-Content (Join-Path $tmp "checksums.txt")) {
            $parts = $line -split "\s+", 2
            if ($parts.Count -eq 2) { $sums[$parts[1].Trim()] = $parts[0].ToLower() }
        }
        foreach ($name in @("VERSION", $cli, $app, "LICENSE", "THIRD-PARTY.md")) {
            Get-Release $name
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

        Write-Host ""
        Write-Host "Sprixel $installed is installed."
        if ($newTerminal) { Write-Host "Open a new terminal so that the 'sprixel' command is found." }
        Write-Host ""
        Write-Host "Next steps:"
        Write-Host "  sprixel doctor            check that everything works"
        Write-Host "  sprixel mcp add claude    let Claude Code use Sprixel (or: opencode)"
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
