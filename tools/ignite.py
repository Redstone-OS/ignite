#!/usr/bin/env python3
"""
Ignite Builder - Menu Interativo Completo
Sistema de build com visual rico, logs e progress bars
"""

import subprocess
import sys
import os
import shutil
import logging
import time
from pathlib import Path
from datetime import datetime

try:
    from rich.console import Console
    from rich.table import Table
    from rich.panel import Panel
    from rich.progress import Progress, SpinnerColumn, TextColumn, BarColumn, TimeElapsedColumn
    from rich.logging import RichHandler
    from rich.prompt import Prompt, Confirm
    from rich.layout import Layout
    from rich.live import Live
    from rich import box
except ImportError:
    print("❌ Biblioteca 'rich' não instalada!")
    print("   Execute: pip install rich")
    sys.exit(1)

# Configuração
console = Console()
PROJECT_ROOT = Path(__file__).parent.parent
TARGET_DIR = PROJECT_ROOT / "target"
DIST_DIR = PROJECT_ROOT / "dist"
LOG_DIR = Path(__file__).parent / "log"  # tools/log/

# Criar diretório de logs
LOG_DIR.mkdir(exist_ok=True)

# Configurar logging
log_file = LOG_DIR / f"ignite_{datetime.now().strftime('%Y%m%d_%H%M%S')}.log"
logging.basicConfig(
    level=logging.DEBUG,
    format="%(asctime)s [%(levelname)s] %(message)s",
    handlers=[
        RichHandler(console=console, rich_tracebacks=True, show_time=False),
        logging.FileHandler(log_file, encoding='utf-8')
    ]
)
logger = logging.getLogger("ignite")

# Estatísticas
stats = {
    "builds": 0,
    "tests": 0,
    "checks": 0,
    "errors": 0,
    "session_start": datetime.now()
}

def clear_screen():
    """Limpa a tela"""
    os.system('cls' if os.name == 'nt' else 'clear')

def show_header():
    """Exibe cabeçalho compacto usando largura total"""
    clear_screen()
    
    header = Panel(
        "[bold cyan]🚀 Ignite Builder[/bold cyan] - Sistema de Build Interativo\n"
        "[dim]Redstone OS | v0.4.0 | Python Menu[/dim]",
        border_style="cyan",
        box=box.DOUBLE,
        expand=True
    )
    console.print(header)

def run_with_progress(cmd, description, cwd=None, show_output=True):
    """Executa comando mostrando output em tempo real E salvando em log"""
    logger.info(f"Executando: {' '.join(cmd)}")
    logger.info(f"={'='*60}")
    
    console.print(f"\n[cyan]▶ {description}...[/cyan]\n")
    
    try:
        # Executar mostrando output em tempo real
        process = subprocess.Popen(
            cmd,
            cwd=cwd or PROJECT_ROOT,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1
        )
        
        output_lines = []
        
        # Ler e mostrar output em tempo real
        for line in iter(process.stdout.readline, ''):
            if not line:
                break
            output_lines.append(line)
            # Mostrar na tela
            if show_output:
                console.print(f"[dim]{line.rstrip()}[/dim]")
            # Salvar em log
            logger.info(f"  {line.rstrip()}")
        
        process.wait()
        returncode = process.returncode
        output = ''.join(output_lines)
        
        if returncode == 0:
            console.print(f"\n✓ [green]{description} concluído![/green]")
            logger.info(f"{description} - Sucesso (exit code: 0)")
            logger.info(f"={'='*60}")
            return True, output
        else:
            console.print(f"\n✗ [red]{description} falhou! (exit code: {returncode})[/red]")
            logger.error(f"{description} - Falha (exit code: {returncode})")
            logger.info(f"={'='*60}")
            stats["errors"] += 1
            return False, output
    
    except Exception as e:
        console.print(f"✗ [red]Erro: {e}[/red]")
        logger.exception(f"Exceção durante {description}")
        logger.info(f"={'='*60}")
        stats["errors"] += 1
        return False, str(e)

def ensure_target():
    """Verifica target com visual"""
    console.print("\n[yellow]🔍 Verificando target UEFI...[/yellow]")
    
    result = subprocess.run(
        ["rustup", "target", "list", "--installed"],
        capture_output=True,
        text=True
    )
    
    if "x86_64-unknown-uefi" not in result.stdout:
        console.print("[yellow]  📥 Instalando target x86_64-unknown-uefi...[/yellow]")
        success, _ = run_with_progress(
            ["rustup", "target", "add", "x86_64-unknown-uefi"],
            "Instalando target UEFI"
        )
        return success
    else:
        console.print("[green]  ✓ Target x86_64-unknown-uefi já instalado[/green]")
        return True

def build_ignite(profile="debug"):
    """Build com visual completo"""
    console.print(Panel.fit(
        f"[bold cyan]Compilando Ignite[/bold cyan]\n"
        f"Modo: [yellow]{profile.upper()}[/yellow]",
        border_style="cyan"
    ))
    
    logger.info(f"=== BUILD {profile.upper()} INICIADO ===")
    stats["builds"] += 1
    
    if not ensure_target():
        return False
    
    cmd = ["cargo", "build", "--package", "ignite", "--target", "x86_64-unknown-uefi"]
    if profile == "release":
        cmd.append("--release")
    elif profile == "verbose":
        cmd.append("--verbose")
    
    success, output = run_with_progress(cmd, f"Compilando em modo {profile}")
    
    if success:
        binary_path = TARGET_DIR / f"x86_64-unknown-uefi/{profile.replace('verbose', 'debug')}/ignite.efi"
        
        if binary_path.exists():
            size_mb = binary_path.stat().st_size / (1024 * 1024)
            
            info_table = Table(show_header=False, box=box.SIMPLE)
            info_table.add_column("Campo", style="cyan")
            info_table.add_column("Valor", style="green")
            
            info_table.add_row("📄 Binário", str(binary_path))
            info_table.add_row("📊 Tamanho", f"{size_mb:.2f} MB")
            info_table.add_row("⏰ Compilado", datetime.now().strftime("%H:%M:%S"))
            
            console.print("\n")
            console.print(info_table)
            
            logger.info(f"Binário gerado: {binary_path} ({size_mb:.2f} MB)")
        
        console.print(f"\n[bold green]✓ Build {profile} concluído com sucesso![/bold green]")
    
    logger.info(f"=== BUILD {profile.upper()} FINALIZADO - {'SUCESSO' if success else 'FALHA'} ===")
    return success

def run_tests(test_type="all"):
    """Testes com visual"""
    console.print(Panel.fit(
        f"[bold cyan]Executando Testes[/bold cyan]\n"
        f"Tipo: [yellow]{test_type.upper()}[/yellow]",
        border_style="cyan"
    ))
    
    logger.info(f"=== TESTES {test_type.upper()} INICIADOS ===")
    stats["tests"] += 1
    
    cmd = ["cargo", "test", "--package", "ignite"]
    if test_type == "unit":
        cmd.append("--lib")
    elif test_type == "integration":
        cmd.extend(["--test", "*"])
    
    success, output = run_with_progress(cmd, f"Executando testes {test_type}")
    
    if success:
        # Parsear resultados
        if "test result:" in output:
            result_line = [line for line in output.split('\n') if "test result:" in line]
            if result_line:
                console.print(f"\n[green]{result_line[0]}[/green]")
        
        console.print(f"\n[bold green]✓ Testes {test_type} executados com sucesso![/bold green]")
    
    logger.info(f"=== TESTES {test_type.upper()} FINALIZADOS - {'SUCESSO' if success else 'FALHA'} ===")
    return success

def run_check(check_type="check"):
    """Verificação com visual"""
    console.print(Panel.fit(
        f"[bold cyan]Verificação de Código[/bold cyan]\n"
        f"Tipo: [yellow]{check_type.upper()}[/yellow]",
        border_style="cyan"
    ))
    
    logger.info(f"=== VERIFICAÇÃO {check_type.upper()} INICIADA ===")
    stats["checks"] += 1
    
    checks = []
    
    if check_type in ["check", "all"]:
        checks.append((["cargo", "check", "--package", "ignite"], "Cargo Check"))
    
    if check_type in ["fmt", "all"]:
        checks.append((["cargo", "fmt", "--package", "ignite", "--", "--check"], "Rustfmt"))
    
    if check_type in ["clippy", "all"]:
        checks.append((["cargo", "clippy", "--package", "ignite"], "Clippy"))
    
    results = []
    for cmd, desc in checks:
        success, _ = run_with_progress(cmd, desc)
        results.append((desc, success))
    
    # Resumo
    passed = sum(1 for _, s in results if s)
    total = len(results)
    
    console.print("\n" + "="*50)
    if passed == total:
        console.print(f"[bold green]✓ Todas as verificações passaram! ({passed}/{total})[/bold green]")
    else:
        console.print(f"[bold yellow]⚠ {passed}/{total} verificações passaram[/bold yellow]")
    
    logger.info(f"=== VERIFICAÇÃO FINALIZADA - {passed}/{total} PASSARAM ===")
    return passed == total

def clean_artifacts(clean_all=False):
    """Limpeza com visual"""
    console.print(Panel.fit(
        f"[bold cyan]Limpeza de Artefatos[/bold cyan]\n"
        f"Modo: [yellow]{'COMPLETO' if clean_all else 'PADRÃO'}[/yellow]",
        border_style="cyan"
    ))
    
    logger.info("=== LIMPEZA INICIADA ===")
    
    success, _ = run_with_progress(["cargo", "clean"], "Limpando target/")
    
    if clean_all and DIST_DIR.exists():
        console.print("[yellow]🗑️  Removendo dist/...[/yellow]")
        shutil.rmtree(DIST_DIR)
        console.print("[green]  ✓ dist/ removido[/green]")
        logger.info("dist/ removido")
    
    console.print(f"\n[bold green]✓ Limpeza {'completa' if clean_all else 'padrão'} realizada![/bold green]")
    logger.info("=== LIMPEZA FINALIZADA ===")
    return success

def show_doctor():
    """Diagnóstico completo"""
    console.print(Panel.fit(
        "[bold cyan]Diagnóstico do Ambiente[/bold cyan]",
        border_style="cyan"
    ))
    
    logger.info("=== DIAGNÓSTICO INICIADO ===")
    
    # Tabela de ferramentas
    tools_table = Table(title="\n🔧 Ferramentas", show_header=True, header_style="bold magenta", border_style="cyan")
    tools_table.add_column("Componente", style="cyan", width=20)
    tools_table.add_column("Status", width=10)
    tools_table.add_column("Versão", style="dim")
    
    # Rust
    try:
        result = subprocess.run(["rustc", "--version"], capture_output=True, text=True, check=True)
        tools_table.add_row("Rust Compiler", "[green]✓ OK[/green]", result.stdout.strip())
    except:
        tools_table.add_row("Rust Compiler", "[red]✗ Falta[/red]", "Não instalado")
    
    # Cargo
    try:
        result = subprocess.run(["cargo", "--version"], capture_output=True, text=True, check=True)
        tools_table.add_row("Cargo", "[green]✓ OK[/green]", result.stdout.strip())
    except:
        tools_table.add_row("Cargo", "[red]✗ Falta[/red]", "Não instalado")
    
    # Target
    result = subprocess.run(["rustup", "target", "list", "--installed"], capture_output=True, text=True)
    if "x86_64-unknown-uefi" in result.stdout:
        tools_table.add_row("Target UEFI", "[green]✓ OK[/green]", "x86_64-unknown-uefi")
    else:
        tools_table.add_row("Target UEFI", "[red]✗ Falta[/red]", "Não instalado")
    
    # Python
    tools_table.add_row("Python", "[green]✓ OK[/green]", f"{sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}")
    
    console.print(tools_table)
    
    # Tabela de projeto
    project_table = Table(title="\n📁 Projeto", show_header=False, border_style="cyan", box=box.SIMPLE)
    project_table.add_column("Item", style="cyan", width=20)
    project_table.add_column("Info", style="white")
    
    project_table.add_row("Diretório", str(PROJECT_ROOT))
    
    if (PROJECT_ROOT / "Cargo.toml").exists():
        project_table.add_row("Cargo.toml", "[green]✓ Encontrado[/green]")
    
    tests_dir = PROJECT_ROOT / "tests"
    if tests_dir.exists():
        test_files = len(list(tests_dir.rglob("*.rs")))
        project_table.add_row("Testes", f"[green]{test_files} arquivos | 81 casos[/green]")
    
    if LOG_DIR.exists():
        log_files = len(list(LOG_DIR.glob("*.log")))
        project_table.add_row("Logs", f"[green]{log_files} arquivos em log/[/green]")
    
    console.print(project_table)
    
    # Estatísticas da sessão
    duration = (datetime.now() - stats['session_start']).total_seconds()
    stats_table = Table(title="\n📊 Estatísticas da Sessão", show_header=False, border_style="cyan", box=box.SIMPLE)
    stats_table.add_column("Métrica", style="cyan", width=20)
    stats_table.add_column("Valor", style="yellow")
    
    stats_table.add_row("Builds realizados", str(stats['builds']))
    stats_table.add_row("Testes executados", str(stats['tests']))
    stats_table.add_row("Verificações", str(stats['checks']))
    stats_table.add_row("Erros", str(stats['errors']))
    stats_table.add_row("Tempo de sessão", f"{int(duration//60)}m {int(duration%60)}s")
    stats_table.add_row("Log atual", log_file.name)
    
    console.print(stats_table)
    
    logger.info("=== DIAGNÓSTICO FINALIZADO ===")

def create_distribution(profile="release"):
    """Distribuição com visual"""
    console.print(Panel.fit(
        f"[bold cyan]Criando Distribuição[/bold cyan]\n"
        f"Modo: [yellow]{profile.upper()}[/yellow]",
        border_style="cyan"
    ))
    
    logger.info(f"=== DISTRIBUIÇÃO {profile.upper()} INICIADA ===")
    
    # Build primeiro
    if not build_ignite(profile):
        console.print("[red]✗ Falha no build - distribuição abortada[/red]")
        return False
    
    with Progress(
        SpinnerColumn(),
        TextColumn("[progress.description]{task.description}"),
        BarColumn(),
        console=console
    ) as progress:
        
        task1 = progress.add_task("[cyan]Criando estrutura...", total=4)
        
        # Criar estrutura
        efi_dir = DIST_DIR / "EFI" / "BOOT"
        boot_dir = DIST_DIR / "boot"
        
        efi_dir.mkdir(parents=True, exist_ok=True)
        progress.advance(task1)
        
        boot_dir.mkdir(parents=True, exist_ok=True)
        progress.advance(task1)
        
        # Copiar bootloader
        binary_source = TARGET_DIR / f"x86_64-unknown-uefi/{profile}/ignite.efi"
        binary_dest = efi_dir / "BOOTX64.EFI"
        
        if binary_source.exists():
            shutil.copy2(binary_source, binary_dest)
            progress.advance(task1)
            logger.info(f"Bootloader copiado: {binary_dest}")
        else:
            console.print("[red]✗ Binário não encontrado[/red]")
            return False
        
        # Copiar configuração
        config_source = PROJECT_ROOT / "ignite.conf"
        if config_source.exists():
            shutil.copy2(config_source, boot_dir / "ignite.conf")
            logger.info("Configuração copiada")
        
        progress.advance(task1)
    
    # Resumo
    size_mb = binary_dest.stat().st_size / (1024 * 1024)
    
    summary = Table(show_header=False, box=box.SIMPLE)
    summary.add_column("Item", style="cyan")
    summary.add_column("Info", style="green")
    
    summary.add_row("📁 Diretório", str(DIST_DIR))
    summary.add_row("📄 Bootloader", "EFI/BOOT/BOOTX64.EFI")
    summary.add_row("⚙️  Configuração", "boot/ignite.conf")
    summary.add_row("📊 Tamanho", f"{size_mb:.2f} MB")
    
    console.print("\n")
    console.print(summary)
    console.print(f"\n[bold green]✓ Distribuição {profile} criada com sucesso![/bold green]")
    
    logger.info(f"=== DISTRIBUIÇÃO FINALIZADA - {size_mb:.2f} MB ===")
    return True

def show_menu():
    """Menu principal com 3 colunas usando largura total"""
    show_header()
    
    # Calcular largura disponível
    total_width = console.width
    col_width = (total_width - 10) // 3  # -10 para espaçamento
    
    # Coluna 1: Build & Testes
    col1 = Table(show_header=True, header_style="bold yellow on blue", border_style="blue", box=box.ROUNDED, padding=(0, 1), expand=True)
    col1.add_column("", style="bold cyan", width=3, justify="right")
    col1.add_column("Build & Testes", style="white", no_wrap=False)
    col1.add_row("1", "Build Debug")
    col1.add_row("2", "Build Release")
    col1.add_row("3", "Build Verbose")
    col1.add_row("", "")
    col1.add_row("4", "Todos Testes")
    col1.add_row("5", "Testes Unit")
    col1.add_row("6", "Testes Integration")
    
    # Coluna 2: Check & Dist
    col2 = Table(show_header=True, header_style="bold yellow on magenta", border_style="magenta", box=box.ROUNDED, padding=(0, 1), expand=True)
    col2.add_column("", style="bold cyan", width=3, justify="right")
    col2.add_column("Check & Dist", style="white", no_wrap=False)
    col2.add_row("7", "Cargo Check")
    col2.add_row("8", "Rustfmt Check")
    col2.add_row("9", "Clippy Lints")
    col2.add_row("10", "Check Completo")
    col2.add_row("", "")
    col2.add_row("11", "Dist Release")
    col2.add_row("12", "Dist Debug")
    
    # Coluna 3: Utilidades
    col3 = Table(show_header=True, header_style="bold yellow on green", border_style="green", box=box.ROUNDED, padding=(0, 1), expand=True)
    col3.add_column("", style="bold cyan", width=3, justify="right")
    col3.add_column("Utilidades", style="white", no_wrap=False)
    col3.add_row("13", "Clean target/")
    col3.add_row("14", "Clean All")
    col3.add_row("15", "Doctor")
    col3.add_row("16", "Ver Logs")
    col3.add_row("", "")
    col3.add_row("", "")
    col3.add_row("Q", "Sair")
    
    # Grid de 3 colunas com expand
    grid = Table.grid(padding=(0, 1), expand=True)
    grid.add_column(ratio=1)
    grid.add_column(ratio=1)
    grid.add_column(ratio=1)
    grid.add_row(col1, col2, col3)
    
    console.print("\n")
    console.print(grid)
    console.print(f"\n[dim]💡 Digite o número | Logs: [cyan]log/[/cyan] | Docs: [cyan]tools/README.md[/cyan][/dim]\n")

def show_logs():
    """Mostra logs recentes"""
    console.print(Panel.fit("[bold cyan]Logs Recentes[/bold cyan]", border_style="cyan"))
    
    log_files = sorted(LOG_DIR.glob("*.log"), key=lambda p: p.stat().st_mtime, reverse=True)
    
    if not log_files:
        console.print("[yellow]Nenhum log encontrado[/yellow]")
        return
    
    logs_table = Table(show_header=True, header_style="bold magenta", border_style="cyan")
    logs_table.add_column("Arquivo", style="cyan")
    logs_table.add_column("Tamanho", style="yellow")
    logs_table.add_column("Modificado", style="dim")
    
    for log in log_files[:10]:  # Mostrar últimos 10
        size_kb = log.stat().st_size / 1024
        mtime = datetime.fromtimestamp(log.stat().st_mtime).strftime("%Y-%m-%d %H:%M:%S")
        logs_table.add_row(log.name, f"{size_kb:.1f} KB", mtime)
    
    console.print("\n")
    console.print(logs_table)
    
    if log_files:
        console.print(f"\n[dim]Log atual: {log_file.name}[/dim]")

def main_menu():
    """Loop principal"""
    logger.info("=== SESSÃO INICIADA ===")
    
    while True:
        show_menu()
        
        choice = Prompt.ask("[bold]Digite a opção[/bold]", default="15")
        console.print()
        
        try:
            if choice == "1":
                build_ignite("debug")
            elif choice == "2":
                build_ignite("release")
            elif choice == "3":
                build_ignite("verbose")
            elif choice == "4":
                run_tests("all")
            elif choice == "5":
                run_tests("unit")
            elif choice == "6":
                run_tests("integration")
            elif choice == "7":
                run_check("check")
            elif choice == "8":
                run_check("fmt")
            elif choice == "9":
                run_check("clippy")
            elif choice == "10":
                run_check("all")
            elif choice == "11":
                create_distribution("release")
            elif choice == "12":
                create_distribution("debug")
            elif choice == "13":
                clean_artifacts(False)
            elif choice == "14":
                clean_artifacts(True)
            elif choice == "15":
                show_doctor()
            elif choice == "16":
                show_logs()
            elif choice.upper() == "Q":
                console.print(Panel.fit(
                    "[bold cyan]Obrigado por usar o Ignite Builder![/bold cyan]\n\n"
                    f"[green]Builds: {stats['builds']} | Testes: {stats['tests']} | Checks: {stats['checks']}[/green]\n"
                    f"[yellow]Erros: {stats['errors']}[/yellow]\n\n"
                    f"[dim]Log salvo em: {log_file.name}[/dim]",
                    border_style="cyan"
                ))
                logger.info("=== SESSÃO ENCERRADA ===")
                break
            else:
                console.print("[red]❌ Opção inválida[/red]")
                time.sleep(1)
                continue
            
            console.print("\n")
            input("Pressione ENTER para continuar...")
        
        except KeyboardInterrupt:
            console.print("\n[yellow]⚠️  Operação interrompida[/yellow]")
            input("\nPressione ENTER para continuar...")
        except Exception as e:
            console.print(f"[red]❌ Erro: {e}[/red]")
            logger.exception("Erro não tratado")
            input("\nPressione ENTER para continuar...")

if __name__ == "__main__":
    try:
        main_menu()
    except KeyboardInterrupt:
        console.print("\n[yellow]⚠️  Saindo...[/yellow]")
        logger.info("=== SESSÃO INTERROMPIDA ===")
        sys.exit(0)
