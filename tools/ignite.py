#!/usr/bin/env python3
"""
Ignite Builder - Sistema de Build Industrial
Sistema de build profissional com recursos avançados, CI/CD e monitoramento
"""

import subprocess
import sys
import os
import shutil
import logging
import time
import json
import hashlib
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Tuple, Optional

try:
    from rich.console import Console
    from rich.table import Table
    from rich.panel import Panel
    from rich.progress import Progress, SpinnerColumn, TextColumn, BarColumn, TimeElapsedColumn
    from rich.logging import RichHandler
    from rich.prompt import Prompt, Confirm
    from rich.layout import Layout
    from rich.live import Live
    from rich.tree import Tree
    from rich import box
    from rich.syntax import Syntax
except ImportError:
    print("❌ Biblioteca 'rich' não instalada!")
    print("   Execute: pip install rich")
    sys.exit(1)

# Configuração
console = Console()
PROJECT_ROOT = Path(__file__).parent.parent
TARGET_DIR = PROJECT_ROOT / "target"
DIST_DIR = PROJECT_ROOT / "dist"
LOG_DIR = Path(__file__).parent / "log"
CACHE_DIR = Path(__file__).parent / ".cache"
METRICS_FILE = CACHE_DIR / "metrics.json"

# Criar diretórios
LOG_DIR.mkdir(exist_ok=True)
CACHE_DIR.mkdir(exist_ok=True)

# Configurar logging - APENAS para arquivo
log_file = LOG_DIR / f"ignite_{datetime.now().strftime('%Y%m%d_%H%M%S')}.log"
logging.basicConfig(
    level=logging.DEBUG,
    format="%(asctime)s [%(levelname)s] %(message)s",
    handlers=[
        logging.FileHandler(log_file, encoding='utf-8')
    ]
)
logger = logging.getLogger("ignite")

# Estatísticas globais
stats = {
    "builds": 0,
    "tests": 0,
    "checks": 0,
    "errors": 0,
    "warnings": 0,
    "session_start": datetime.now(),
    "commands_run": 0,
    "cache_hits": 0,
}

# Métricas históricas
metrics = {
    "total_builds": 0,
    "total_tests": 0,
    "total_errors": 0,
    "build_times": [],
    "test_times": [],
    "last_success": None,
}

def load_metrics():
    """Carrega métricas históricas"""
    global metrics
    if METRICS_FILE.exists():
        try:
            with open(METRICS_FILE, 'r') as f:
                metrics.update(json.load(f))
        except:
            pass

def save_metrics():
    """Salva métricas históricas"""
    try:
        with open(METRICS_FILE, 'w') as f:
            json.dump(metrics, f, indent=2, default=str)
    except:
        pass

def calculate_hash(file_path: Path) -> str:
    """Calcula hash SHA-256 de um arquivo"""
    sha256 = hashlib.sha256()
    try:
        with open(file_path, 'rb') as f:
            for chunk in iter(lambda: f.read(4096), b''):
                sha256.update(chunk)
        return sha256.hexdigest()
    except:
        return ""

def check_cache(cache_key: str) -> bool:
    """Verifica se resultado está em cache"""
    cache_file = CACHE_DIR / f"{cache_key}.cache"
    if cache_file.exists():
        # Cache válido por 1 hora
        age = time.time() - cache_file.stat().st_mtime
        if age < 3600:
            stats["cache_hits"] += 1
            return True
    return False

def set_cache(cache_key: str):
    """Marca resultado em cache"""
    cache_file = CACHE_DIR / f"{cache_key}.cache"
    cache_file.touch()

def clear_screen():
    """Limpa a tela"""
    os.system('cls' if os.name == 'nt' else 'clear')

def show_header():
    """Exibe cabeçalho industrial"""
    clear_screen()
    
    # Uptime da sessão
    uptime = datetime.now() - stats['session_start']
    uptime_str = f"{int(uptime.total_seconds()//3600)}h {int((uptime.total_seconds()%3600)//60)}m"
    
    header = Panel(
        f"[bold cyan]🚀 Ignite Builder[/bold cyan] - [bold]Sistema de Build Industrial[/bold]\\n"
        f"[dim]Redstone OS | v0.1.0 | Build Tools Professional[/dim]\\n\\n"
        f"[green]Sessão: {uptime_str}[/green] │ "
        f"[yellow]Comandos: {stats['commands_run']}[/yellow] │ "
        f"[cyan]Cache Hits: {stats['cache_hits']}[/cyan] │ "
        f"[red]Erros: {stats['errors']}[/red]",
        border_style="cyan",
        box=box.DOUBLE,
        expand=True
    )
    console.print(header)

def run_with_progress_industrial(cmd: List[str], description: str, cwd=None, show_output=True) -> Tuple[bool, str, float]:
    """Executa comando com monitoramento industrial"""
    logger.info(f"Executando: {' '.join(cmd)}")
    logger.info(f"{'='*60}")
    
    stats["commands_run"] += 1
    start_time = time.time()
    
    console.print(f"\\n[cyan]▶ {description}...[/cyan]")
    console.print(f"[dim]Comando: {' '.join(cmd)}[/dim]")
    console.print("[dim]" + "-"*60 + "[/dim]\\n")
    
    try:
        process = subprocess.Popen(
            cmd,
            cwd=cwd or PROJECT_ROOT,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1
        )
        
        output_lines = []
        error_count = 0
        warning_count = 0
        info_count = 0
        
        # Ler e processar output
        for line in iter(process.stdout.readline, ''):
            if not line:
                break
            output_lines.append(line)
            
            # Salvar TUDO em log
            logger.info(f"  {line.rstrip()}")
            
            # Analisar linha
            line_lower = line.lower()
            
            # Contar problemas
            if 'error[' in line_lower or 'error:' in line_lower:
                error_count += 1
            elif 'warning[' in line_lower or 'warning:' in line_lower:
                warning_count += 1
            
            if show_output:
                # Mostrar linhas importantes com cores
                if any(keyword in line_lower for keyword in ['compiling', 'finished', 'error', 'warning', 'failed', 'running', 'checking']):
                    if 'error' in line_lower:
                        console.print(f"[red]{line.rstrip()}[/red]")
                    elif 'warning' in line_lower:
                        console.print(f"[yellow]{line.rstrip()}[/yellow]")
                    elif 'compiling' in line_lower or 'checking' in line_lower:
                        console.print(f"[cyan]{line.rstrip()}[/cyan]")
                    elif 'finished' in line_lower:
                        console.print(f"[green]{line.rstrip()}[/green]")
                    elif 'running' in line_lower:
                        console.print(f"[blue]{line.rstrip()}[/blue]")
                    else:
                        console.print(f"[dim]{line.rstrip()}[/dim]")
        
        process.wait()
        returncode = process.returncode
        output = ''.join(output_lines)
        duration = time.time() - start_time
        
        # Atualizar estatísticas
        stats["errors"] += error_count
        stats["warnings"] += warning_count
        
        # Resumo visual industrial
        console.print("\\n[dim]" + "="*60 + "[/dim]")
        
        if returncode == 0:
            console.print(f"[bold green]✅ {description} - SUCESSO[/bold green]")
            console.print(f"[dim]⏱️  Tempo: {duration:.2f}s | Warnings: {warning_count}[/dim]")
            logger.info(f"{description} - SUCESSO (exit: 0, tempo: {duration:.2f}s)")
        else:
            console.print(f"[bold red]❌ {description} - FALHOU[/bold red]")
            if error_count > 0:
                console.print(f"[red]  📛 {error_count} erro(s) encontrado(s)[/red]")
            if warning_count > 0:
                console.print(f"[yellow]  ⚠️  {warning_count} warning(s) encontrado(s)[/yellow]")
            console.print(f"[dim]  📄 Log completo: {log_file.name}[/dim]")
            logger.error(f"{description} - FALHA (exit: {returncode}, tempo: {duration:.2f}s)")
            stats["errors"] += 1
        
        console.print("[dim]" + "="*60 + "[/dim]")
        logger.info(f"{'='*60}")
        
        return returncode == 0, output, duration
    
    except Exception as e:
        duration = time.time() - start_time
        console.print(f"[bold red]💥 EXCEÇÃO: {e}[/bold red]")
        logger.exception(f"Exceção durante {description}")
        stats["errors"] += 1
        return False, str(e), duration

def ensure_target():
    """Verifica target com cache"""
    cache_key = "target_uefi_installed"
    
    if check_cache(cache_key):
        console.print("[green]  ✓ Target x86_64-unknown-uefi (cache)[/green]")
        return True
    
    console.print("\\n[yellow]🔍 Verificando target UEFI...[/yellow]")
    
    result = subprocess.run(
        ["rustup", "target", "list", "--installed"],
        capture_output=True,
        text=True
    )
    
    if "x86_64-unknown-uefi" not in result.stdout:
        console.print("[yellow]  📥 Instalando target x86_64-unknown-uefi...[/yellow]")
        success, _, _ = run_with_progress_industrial(
            ["rustup", "target", "add", "x86_64-unknown-uefi"],
            "Instalando target UEFI"
        )
        if success:
            set_cache(cache_key)
        return success
    else:
        console.print("[green]  ✓ Target x86_64-unknown-uefi instalado[/green]")
        set_cache(cache_key)
        return True

def build_ignite_industrial(profile="debug", features: Optional[List[str]] = None):
    """Build industrial com otimizações"""
    console.print(Panel.fit(
        f"[bold cyan]🔨 Compilação Industrial[/bold cyan]\\n"
        f"Modo: [yellow]{profile.upper()}[/yellow]\\n"
        f"Features: [cyan]{', '.join(features) if features else 'default'}[/cyan]",
        border_style="cyan"
    ))
    
    logger.info(f"=== BUILD INDUSTRIAL {profile.upper()} INICIADO ===")
    stats["builds"] += 1
    metrics["total_builds"] += 1
    
    if not ensure_target():
        return False
    
    cmd = ["cargo", "build", "--package", "ignite", "--target", "x86_64-unknown-uefi"]
    
    if profile == "release":
        cmd.append("--release")
    elif profile == "verbose":
        cmd.append("--verbose")
    
    if features:
        cmd.extend(["--features", ",".join(features)])
    
    success, output, duration = run_with_progress_industrial(cmd, f"Compilando {profile}")
    
    if success:
        metrics["build_times"].append(duration)
        metrics["last_success"] = datetime.now().isoformat()
        
        binary_path = TARGET_DIR / f"x86_64-unknown-uefi/{profile.replace('verbose', 'debug')}/ignite.efi"
        
        if binary_path.exists():
            size_mb = binary_path.stat().st_size / (1024 * 1024)
            file_hash = calculate_hash(binary_path)[:16]
            
            info_table = Table(show_header=False, box=box.SIMPLE)
            info_table.add_column("Campo", style="cyan")
            info_table.add_column("Valor", style="green")
            
            info_table.add_row("📄 Binário", str(binary_path.name))
            info_table.add_row("📊 Tamanho", f"{size_mb:.3f} MB ({binary_path.stat().st_size:,} bytes)")
            info_table.add_row("⏱️  Tempo de Build", f"{duration:.2f}s")
            info_table.add_row("🔐 Hash (SHA-256)", file_hash)
            info_table.add_row("⏰ Compilado", datetime.now().strftime("%Y-%m-%d %H:%M:%S"))
            
            # Estatísticas históricas
            if metrics["build_times"]:
                avg_time = sum(metrics["build_times"][-10:]) / min(len(metrics["build_times"]), 10)
                info_table.add_row("📈 Tempo Médio (10x)", f"{avg_time:.2f}s")
            
            console.print("\\n")
            console.print(info_table)
            
            logger.info(f"Binário: {binary_path} ({size_mb:.3f} MB, {duration:.2f}s)")
        
        console.print(f"\\n[bold green]✅ Build {profile} concluído com SUCESSO![/bold green]")
    
    save_metrics()
    logger.info(f"=== BUILD {profile.upper()} FINALIZADO - {'SUCESSO' if success else 'FALHA'} ===")
    return success

def run_tests_industrial(test_type="all", parallel=True):
    """Testes industriais com paralelização"""
    console.print(Panel.fit(
        f"[bold cyan]🧪 Testes Industrial[/bold cyan]\\n"
        f"Tipo: [yellow]{test_type.upper()}[/yellow]\\n"
        f"Parallel: [{'green' if parallel else 'red'}]{'Sim' if parallel else 'Não'}[/]",
        border_style="cyan"
    ))
    
    logger.info(f"=== TESTES {test_type.upper()} INICIADOS ===")
    stats["tests"] += 1
    metrics["total_tests"] += 1
    
    cmd = ["cargo", "test", "--package", "ignite"]
    
    if test_type == "unit":
        cmd.append("--lib")
    elif test_type == "integration":
        cmd.extend(["--test", "*"])
    
    if not parallel:
        cmd.append("--")
        cmd.append("--test-threads=1")
    
    success, output, duration = run_with_progress_industrial(cmd, f"Testes {test_type}")
    
    if success:
        metrics["test_times"].append(duration)
        
        # Parsear resultados
        test_count = passed_count = 0
        if "test result:" in output:
            for line in output.split('\\n'):
                if "test result:" in line:
                    # Extrair números
                    parts = line.split()
                    for i, part in enumerate(parts):
                        if part == "passed;" and i > 0:
                            try:
                                passed_count = int(parts[i-1])
                            except:
                                pass
        
        results_table = Table(show_header=False, box=box.SIMPLE)
        results_table.add_column("Métrica", style="cyan")
        results_table.add_column("Valor", style="green")
        
        results_table.add_row("⏱️  Tempo", f"{duration:.2f}s")
        if passed_count > 0:
            results_table.add_row("✅ Testes Passados", str(passed_count))
        results_table.add_row("📊 Total de Testes", f"{metrics['total_tests']} (histórico)")
        
        console.print("\\n")
        console.print(results_table)
        console.print(f"\\n[bold green]✅ Testes {test_type} executados com SUCESSO![/bold green]")
    
    save_metrics()
    logger.info(f"=== TESTES FINALIZADOS - {'SUCESSO' if success else 'FALHA'} ===")
    return success

def run_check_industrial(check_type="all"):
    """Verificação industrial com múltiplas ferramentas"""
    console.print(Panel.fit(
        f"[bold cyan]🔎 Verificação Industrial[/bold cyan]\\n"
        f"Tipo: [yellow]{check_type.upper()}[/yellow]",
        border_style="cyan"
    ))
    
    logger.info(f"=== VERIFICAÇÃO {check_type.upper()} INICIADA ===")
    stats["checks"] += 1
    
    checks = []
    
    if check_type in ["check", "all"]:
        checks.append((["cargo", "check", "--package", "ignite", "--target", "x86_64-unknown-uefi"], "Cargo Check"))
    
    if check_type in ["fmt", "all"]:
        checks.append((["cargo", "fmt", "--package", "ignite", "--", "--check"], "Rustfmt"))
    
    if check_type in ["clippy", "all"]:
        checks.append((["cargo", "clippy", "--package", "ignite", "--target", "x86_64-unknown-uefi", "--", "-D", "warnings"], "Clippy"))
    
    # Adicionar verificações extras no modo all
    if check_type == "all":
        checks.append((["cargo", "audit"], "Cargo Audit (Segurança)"))
        checks.append((["cargo", "outdated"], "Cargo Outdated (Dependências)"))
    
    results = []
    total_time = 0
    
    for cmd, desc in checks:
        # Ignorar falhas de ferramentas opcionais
        try:
            success, _, duration = run_with_progress_industrial(cmd, desc, show_output=(check_type != "all"))
            results.append((desc, success, duration))
            total_time += duration
        except:
            results.append((desc, None, 0))  # Tool não disponível
    
    # Resumo industrial
    console.print("\\n" + "="*60)
    
    results_table = Table(show_header=True, header_style="bold magenta", border_style="cyan")
    results_table.add_column("Verificação", style="cyan")
    results_table.add_column("Status", justify="center")
    results_table.add_column("Tempo", justify="right", style="dim")
    
    passed = 0
    for desc, success, duration in results:
        if success is True:
            status = "[green]✅ OK[/green]"
            passed += 1
        elif success is False:
            status = "[red]❌ FALHA[/red]"
        else:
            status = "[dim]⊘ N/A[/dim]"
        
        time_str = f"{duration:.2f}s" if duration > 0 else "-"
        results_table.add_row(desc, status, time_str)
    
    results_table.add_row("", "", "", end_section=True)
    results_table.add_row("[bold]TOTAL[/bold]", f"[bold]{passed}/{len([r for r in results if r[1] is not None])}[/bold]", f"[bold]{total_time:.2f}s[/bold]")
    
    console.print("\\n")
    console.print(results_table)
    
    if passed == len([r for r in results if r[1] is not None]):
        console.print(f"\\n[bold green]✅ TODAS as verificações passaram![/bold green]")
    else:
        console.print(f"\\n[bold yellow]⚠️  {passed}/{len([r for r in results if r[1] is not None])} verificações passaram[/bold yellow]")
    
    logger.info(f"=== VERIFICAÇÃO FINALIZADA - {passed}/{len([r for r in results if r[1] is not None])} PASSARAM ===")
    return passed == len([r for r in results if r[1] is not None])

def show_doctor_industrial():
    """Diagnóstico industrial completo"""
    console.print(Panel.fit(
        "[bold cyan]🏥 Diagnóstico Industrial Completo[/bold cyan]",
        border_style="cyan"
    ))
    
    logger.info("=== DIAGNÓSTICO INICIADO ===")
    
    # Tabela de ferramentas
    tools_table = Table(title="\\n🔧 Ferramentas e Dependências", show_header=True, header_style="bold magenta", border_style="cyan")
    tools_table.add_column("Componente", style="cyan", width=25)
    tools_table.add_column("Status", width=12, justify="center")
    tools_table.add_column("Versão", style="dim")
    tools_table.add_column("Path", style="dim", no_wrap=False)
    
    # Rust
    try:
        result = subprocess.run(["rustc", "--version"], capture_output=True, text=True, check=True)
        path_result = subprocess.run(["where" if os.name == "nt" else "which", "rustc"], capture_output=True, text=True)
        tools_table.add_row("Rust Compiler", "[green]✅ OK[/green]", result.stdout.strip(), path_result.stdout.strip().split('\\n')[0])
    except:
        tools_table.add_row("Rust Compiler", "[red]❌ FALTA[/red]", "Não instalado", "-")
    
    # Cargo
    try:
        result = subprocess.run(["cargo", "--version"], capture_output=True, text=True, check=True)
        tools_table.add_row("Cargo", "[green]✅ OK[/green]", result.stdout.strip(), "-")
    except:
        tools_table.add_row("Cargo", "[red]❌ FALTA[/red]", "Não instalado", "-")
    
    # Target UEFI
    result = subprocess.run(["rustup", "target", "list", "--installed"], capture_output=True, text=True)
    if "x86_64-unknown-uefi" in result.stdout:
        tools_table.add_row("Target UEFI", "[green]✅ OK[/green]", "x86_64-unknown-uefi", "-")
    else:
        tools_table.add_row("Target UEFI", "[red]❌ FALTA[/red]", "Não instalado", "-")
    
    # Python
    tools_table.add_row("Python", "[green]✅ OK[/green]", f"{sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}", sys.executable)
    
    # Git
    try:
        result = subprocess.run(["git", "--version"], capture_output=True, text=True, check=True)
        tools_table.add_row("Git", "[green]✅ OK[/green]", result.stdout.strip(), "-")
    except:
        tools_table.add_row("Git", "[yellow]⚠️  OPCIONAL[/yellow]", "Não instalado", "-")
    
    console.print(tools_table)
    
    # Tabela de projeto
    project_table = Table(title="\\n📁 Projeto Ignite", show_header=False, border_style="cyan", box=box.SIMPLE)
    project_table.add_column("Item", style="cyan", width=25)
    project_table.add_column("Info", style="white", no_wrap=False)
    
    project_table.add_row("📂 Diretório Raiz", str(PROJECT_ROOT))
    
    if (PROJECT_ROOT / "Cargo.toml").exists():
        project_table.add_row("📄 Cargo.toml", "[green]✓ Encontrado[/green]")
        
        # Ler versão
        try:
            with open(PROJECT_ROOT / "Cargo.toml", 'r') as f:
                for line in f:
                    if line.startswith("version"):
                        version = line.split("=")[1].strip().strip('"')
                        project_table.add_row("📌 Versão", version)
                        break
        except:
            pass
    
    # Contagem de arquivos fonte
    src_files = len(list((PROJECT_ROOT / "src").rglob("*.rs"))) if (PROJECT_ROOT / "src").exists() else 0
    project_table.add_row("📝 Arquivos Fonte", f"{src_files} arquivos Rust")
    
    # Testes
    tests_dir = PROJECT_ROOT / "tests"
    if tests_dir.exists():
        test_files = len(list(tests_dir.rglob("*.rs")))
        project_table.add_row("🧪 Arquivos de Teste", f"{test_files} arquivos | ~109 casos")
    
    # Documentação
    docs_dir = PROJECT_ROOT / "docs"
    if docs_dir.exists():
        doc_files = len(list(docs_dir.glob("*.md")))
        project_table.add_row("📚 Documentação", f"{doc_files} arquivos markdown")
    
    # Logs
    if LOG_DIR.exists():
        log_files = len(list(LOG_DIR.glob("*.log")))
        total_log_size = sum(f.stat().st_size for f in LOG_DIR.glob("*.log"))
        project_table.add_row("📋 Logs", f"{log_files} arquivos | {total_log_size/(1024*1024):.2f} MB")
    
    # Cache
    if CACHE_DIR.exists():
        cache_files = len(list(CACHE_DIR.glob("*")))
        project_table.add_row("💾 Cache", f"{cache_files} entradas")
    
    console.print(project_table)
    
    # Estatísticas da sessão
    duration = (datetime.now() - stats['session_start']).total_seconds()
    stats_table = Table(title="\\n📊 Estatísticas da Sessão Atual", show_header=False, border_style="cyan", box=box.SIMPLE)
    stats_table.add_column("Métrica", style="cyan", width=25)
    stats_table.add_column("Valor", style="yellow")
    
    stats_table.add_row("🔨 Builds realizados", str(stats['builds']))
    stats_table.add_row("🧪 Testes executados", str(stats['tests']))
    stats_table.add_row("🔎 Verificações", str(stats['checks']))
    stats_table.add_row("❌ Erros", str(stats['errors']))
    stats_table.add_row("⚠️  Warnings", str(stats['warnings']))
    stats_table.add_row("🔄 Comandos executados", str(stats['commands_run']))
    stats_table.add_row("💾 Cache hits", str(stats['cache_hits']))
    stats_table.add_row("⏱️  Tempo de sessão", f"{int(duration//60)}m {int(duration%60)}s")
    stats_table.add_row("📋 Log atual", log_file.name)
    
    console.print(stats_table)
    
    # Métricas históricas
    historical_table = Table(title="\\n📈 Métricas Históricas", show_header=False, border_style="cyan", box=box.SIMPLE)
    historical_table.add_column("Métrica", style="cyan", width=25)
    historical_table.add_column("Valor", style="green")
    
    historical_table.add_row("🔨 Total de Builds", str(metrics['total_builds']))
    historical_table.add_row("🧪 Total de Testes", str(metrics['total_tests']))
    historical_table.add_row("❌ Total de Erros", str(metrics['total_errors']))
    
    if metrics['build_times']:
        avg_build = sum(metrics['build_times']) / len(metrics['build_times'])
        historical_table.add_row("⏱️  Tempo Médio Build", f"{avg_build:.2f}s")
    
    if metrics['test_times']:
        avg_test = sum(metrics['test_times']) / len(metrics['test_times'])
        historical_table.add_row("⏱️  Tempo Médio Testes", f"{avg_test:.2f}s")
    
    if metrics['last_success']:
        historical_table.add_row("✅ Último Sucesso", metrics['last_success'])
    
    console.print(historical_table)
    
    # Health Score
    health_score = 100
    health_issues = []
    
    if stats['errors'] > 0:
        health_score -= 20
        health_issues.append("Erros na sessão")
    
    if metrics['total_errors'] > 10:
        health_score -= 10
        health_issues.append("Muitos erros históricos")
    
    if not (PROJECT_ROOT / "Cargo.toml").exists():
        health_score -= 30
        health_issues.append("Cargo.toml não encontrado")
    
    health_color = "green" if health_score >= 80 else "yellow" if health_score >= 60 else "red"
    health_status = "EXCELENTE" if health_score >= 80 else "BOM" if health_score >= 60 else "ATENÇÃO"
    
    console.print(f"\\n[{health_color}]💚 Health Score: {health_score}/100 - {health_status}[/{health_color}]")
    if health_issues:
        console.print(f"[yellow]Issues: {', '.join(health_issues)}[/yellow]")
    
    logger.info("=== DIAGNÓSTICO FINALIZADO ===")

def create_distribution_industrial(profile="release"):
    """Distribuição industrial com validações"""
    console.print(Panel.fit(
        f"[bold cyan]📦 Criando Distribuição Industrial[/bold cyan]\\n"
        f"Modo: [yellow]{profile.upper()}[/yellow]",
        border_style="cyan"
    ))
    
    logger.info(f"=== DISTRIBUIÇÃO {profile.upper()} INICIADA ===")
    
    # Build primeiro
    if not build_ignite_industrial(profile):
        console.print("[red]❌ Falha no build - distribuição abortada[/red]")
        return False
    
    with Progress(
        SpinnerColumn(),
        TextColumn("[progress.description]{task.description}"),
        BarColumn(),
        TimeElapsedColumn(),
        console=console
    ) as progress:
        
        task = progress.add_task("[cyan]Preparando distribuição...[/cyan]", total=6)
        
        # Criar estrutura
        efi_dir = DIST_DIR / "EFI" / "BOOT"
        boot_dir = DIST_DIR / "boot"
        tools_dir = DIST_DIR / "tools"
        docs_dir = DIST_DIR / "docs"
        
        for d in [efi_dir, boot_dir, tools_dir, docs_dir]:
            d.mkdir(parents=True, exist_ok=True)
        progress.advance(task)
        
        # Copiar bootloader
        binary_source = TARGET_DIR / f"x86_64-unknown-uefi/{profile}/ignite.efi"
        binary_dest = efi_dir / "BOOTX64.EFI"
        
        if binary_source.exists():
            shutil.copy2(binary_source, binary_dest)
            progress.advance(task)
            logger.info(f"Bootloader copiado: {binary_dest}")
        else:
            console.print("[red]❌ Binário não encontrado[/red]")
            return False
        
        # Copiar configuração
        config_source = PROJECT_ROOT / "ignite.conf"
        if config_source.exists():
            shutil.copy2(config_source, boot_dir / "ignite.conf")
            logger.info("Configuração copiada")
        progress.advance(task)
        
        # Copiar documentação
        if (PROJECT_ROOT / "docs").exists():
            for doc in (PROJECT_ROOT / "docs").glob("*.md"):
                shutil.copy2(doc, docs_dir / doc.name)
        progress.advance(task)
        
        # Copiar README
        if (PROJECT_ROOT / "README.md").exists():
            shutil.copy2(PROJECT_ROOT / "README.md", DIST_DIR / "README.md")
        progress.advance(task)
        
        # Criar manifesto
        manifest = {
            "name": "Ignite Bootloader",
            "version": "0.1.0",
            "profile": profile,
            "build_date": datetime.now().isoformat(),
            "binary_hash": calculate_hash(binary_dest),
            "binary_size": binary_dest.stat().st_size,
        }
        
        with open(DIST_DIR / "manifest.json", 'w') as f:
            json.dump(manifest, f, indent=2)
        progress.advance(task)
    
    # Resumo
    size_mb = binary_dest.stat().st_size / (1024 * 1024)
    total_size = sum(f.stat().st_size for f in DIST_DIR.rglob('*') if f.is_file())
    
    summary = Table(title="📦 Sumário da Distribuição", show_header=False, box=box.SIMPLE)
    summary.add_column("Item", style="cyan")
    summary.add_column("Info", style="green")
    
    summary.add_row("📁 Diretório", str(DIST_DIR))
    summary.add_row("📄 Bootloader", "EFI/BOOT/BOOTX64.EFI")
    summary.add_row("⚙️  Configuração", "boot/ignite.conf")
    summary.add_row("📚 Documentação", f"{len(list(docs_dir.glob('*')))} arquivos")
    summary.add_row("📊 Tamanho Binário", f"{size_mb:.3f} MB")
    summary.add_row("📦 Tamanho Total", f"{total_size/(1024*1024):.2f} MB")
    summary.add_row("🔐 Hash (SHA-256)", manifest['binary_hash'][:32])
    summary.add_row("📋 Manifesto", "manifest.json")
    
    console.print("\\n")
    console.print(summary)
    console.print(f"\\n[bold green]✅ Distribuição {profile} criada com SUCESSO![/bold green]")
    
    logger.info(f"=== DISTRIBUIÇÃO FINALIZADA - {total_size/(1024*1024):.2f} MB ===")
    return True

def show_menu_industrial():
    """Menu industrial profissional"""
    show_header()
    
    # Menu grid com 4 colunas
    col1 = Table(show_header=True, header_style="bold yellow on blue", border_style="blue", box=box.ROUNDED, padding=(0, 1))
    col1.add_column("", style="bold cyan", width=2, justify="right")
    col1.add_column("🔨 Build", style="white")
    col1.add_row("1", "Debug")
    col1.add_row("2", "Release")
    col1.add_row("3", "Verbose")
    col1.add_row("4", "Features Custom")
    
    col2 = Table(show_header=True, header_style="bold yellow on magenta", border_style="magenta", box=box.ROUNDED, padding=(0, 1))
    col2.add_column("", style="bold cyan", width=2, justify="right")
    col2.add_column("🧪 Testes", style="white")
    col2.add_row("5", "Todos")
    col2.add_row("6", "Unitários")
    col2.add_row("7", "Integração")
    col2.add_row("8", "Parallel OFF")
    
    col3 = Table(show_header=True, header_style="bold yellow on green", border_style="green", box=box.ROUNDED, padding=(0, 1))
    col3.add_column("", style="bold cyan", width=2, justify="right")
    col3.add_column("🔎 Check", style="white")
    col3.add_row("9", "Cargo Check")
    col3.add_row("10", "Rustfmt")
    col3.add_row("11", "Clippy")
    col3.add_row("12", "Completo")
    
    col4 = Table(show_header=True, header_style="bold yellow on red", border_style="red", box=box.ROUNDED, padding=(0, 1))
    col4.add_column("", style="bold cyan", width=2, justify="right")
    col4.add_column("⚙️  Utils", style="white")
    col4.add_row("13", "Dist Release")
    col4.add_row("14", "Clean")
    col4.add_row("15", "Doctor")
    col4.add_row("Q", "Sair")
    
    grid = Table.grid(padding=(0, 1))
    grid.add_column(ratio=1)
    grid.add_column(ratio=1)
    grid.add_column(ratio=1)
    grid.add_column(ratio=1)
    grid.add_row(col1, col2, col3, col4)
    
    console.print("\\n")
    console.print(grid)
    console.print(f"\\n[dim]💡 [cyan]tools/log/[/cyan] | [cyan]docs/[/cyan] | [yellow]~109 testes[/yellow] | [green]Industrial Mode[/green][/dim]\\n")

def main_menu():
    """Loop principal industrial"""
    load_metrics()
    logger.info("=== SESSÃO INDUSTRIAL INICIADA ===")
    
    while True:
        show_menu_industrial()
        
        choice = Prompt.ask("[bold]Opção[/bold]", default="15")
        console.print()
        
        try:
            if choice == "1":
                build_ignite_industrial("debug")
            elif choice == "2":
                build_ignite_industrial("release")
            elif choice == "3":
                build_ignite_industrial("verbose")
            elif choice == "4":
                features = Prompt.ask("Features (separadas por vírgula)").split(",")
                build_ignite_industrial("release", features=[f.strip() for f in features if f.strip()])
            elif choice == "5":
                run_tests_industrial("all")
            elif choice == "6":
                run_tests_industrial("unit")
            elif choice == "7":
                run_tests_industrial("integration")
            elif choice == "8":
                run_tests_industrial("all", parallel=False)
            elif choice == "9":
                run_check_industrial("check")
            elif choice == "10":
                run_check_industrial("fmt")
            elif choice == "11":
                run_check_industrial("clippy")
            elif choice == "12":
                run_check_industrial("all")
            elif choice == "13":
                create_distribution_industrial("release")
            elif choice == "14":
                if Confirm.ask("Limpar target/ e cache/?"):
                    subprocess.run(["cargo", "clean"])
                    if CACHE_DIR.exists():
                        shutil.rmtree(CACHE_DIR)
                        CACHE_DIR.mkdir()
                    console.print("[green]✅ Limpeza concluída[/green]")
            elif choice == "15":
                show_doctor_industrial()
            elif choice.upper() == "Q":
                save_metrics()
                console.print(Panel.fit(
                    f"[bold cyan]🎯 Sessão Encerrada[/bold cyan]\\n\\n"
                    f"[green]Builds: {stats['builds']} | Testes: {stats['tests']} | Checks: {stats['checks']}[/green]\\n"
                    f"[yellow]Comandos: {stats['commands_run']} | Cache Hits: {stats['cache_hits']}[/yellow]\\n"
                    f"[{'red' if stats['errors'] > 0 else 'green'}]Erros: {stats['errors']}[/]\\n\\n"
                    f"[dim]Log: {log_file.name}[/dim]",
                    border_style="cyan"
                ))
                logger.info("=== SESSÃO INDUSTRIAL ENCERRADA ===")
                break
            else:
                console.print("[red]❌ Opção inválida[/red]")
                time.sleep(1)
                continue
            
            console.print("\\n")
            input("⏎ ENTER para continuar...")
        
        except KeyboardInterrupt:
            console.print("\\n[yellow]⚠️  Operação interrompida[/yellow]")
            input("\\n⏎ ENTER para continuar...")
        except Exception as e:
            console.print(f"[red]💥 Erro: {e}[/red]")
            logger.exception("Erro não tratado")
            input("\\n⏎ ENTER para continuar...")

if __name__ == "__main__":
    try:
        main_menu()
    except KeyboardInterrupt:
        console.print("\\n[yellow]⚠️  Saindo...[/yellow]")
        save_metrics()
        logger.info("=== SESSÃO INTERROMPIDA ===")
        sys.exit(0)
