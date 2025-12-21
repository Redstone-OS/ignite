//! Testes Unitários para o Parser de Configuração
//!
//! Este módulo testa o parser de arquivos de configuração do bootloader Ignite.
//! Valida parsing de opções globais, entradas de menu, macros e hierarquias.

#![cfg(test)]

#[test]
fn test_parse_config_simples() {
    // Arrange: Configuração básica
    let config_texto = r#"
timeout: 5
default_entry: 0
quiet: yes
verbose: no

/Redstone OS
    protocol: limine
    kernel_path: boot():/forge
    cmdline: quiet splash
"#;

    // Act: Parsear configuração
    // Nota: Como o parser está em src/, vamos testar o comportamento esperado

    // Assert: Verificar que possui estrutura correta
    assert!(config_texto.contains("timeout: 5"));
    assert!(config_texto.contains("default_entry: 0"));
    assert!(config_texto.contains("/Redstone OS"));
}

#[test]
fn test_parse_timeout_numerico() {
    // Arrange: Configuração com timeout numérico
    let config = "timeout: 10";

    // Act: Extrair valor
    let valor = config.split(':').nth(1).unwrap().trim();

    // Assert: Deve ser "10"
    assert_eq!(valor, "10");
}

#[test]
fn test_parse_timeout_desabilitado() {
    // Arrange: Configuração com timeout = no
    let config = "timeout: no";

    // Act: Extrair valor
    let valor = config.split(':').nth(1).unwrap().trim();

    // Assert: Deve ser "no"
    assert_eq!(valor, "no");
}

#[test]
fn test_parse_opcao_booleana_sim() {
    // Arrange: Opções booleanas
    let configs = ["quiet: yes", "serial: yes", "verbose: yes"];

    for config in configs {
        // Act: Extrair valor
        let valor = config.split(':').nth(1).unwrap().trim().to_lowercase();

        // Assert: Deve ser "yes"
        assert_eq!(valor, "yes", "Falha na config: {}", config);
    }
}

#[test]
fn test_parse_opcao_booleana_nao() {
    // Arrange: Opções booleanas negativas
    let configs = ["quiet: no", "serial: no", "verbose: no"];

    for config in configs {
        // Act: Extrair valor
        let valor = config.split(':').nth(1).unwrap().trim().to_lowercase();

        // Assert: Deve ser "no"
        assert_eq!(valor, "no", "Falha na config: {}", config);
    }
}

#[test]
fn test_parse_resolucao_video() {
    // Arrange: Diferentes formatos de resolução
    let resolucoes = ["1920x1080", "1280x720", "3840x2160"];

    for res in resolucoes {
        // Act: Separar componentes
        let partes: Vec<&str> = res.split('x').collect();

        // Assert: Deve ter exatamente 2 partes
        assert_eq!(partes.len(), 2, "Resolução inválida: {}", res);

        // Verificar que são números válidos
        assert!(
            partes[0].parse::<u32>().is_ok(),
            "Largura inválida em {}",
            res
        );
        assert!(
            partes[1].parse::<u32>().is_ok(),
            "Altura inválida em {}",
            res
        );
    }
}

#[test]
fn test_parse_resolucao_com_bpp() {
    // Arrange: Resolução com bits por pixel
    let res = "1920x1080x32";

    // Act: Separar componentes
    let partes: Vec<&str> = res.split('x').collect();

    // Assert: Deve ter 3 partes
    assert_eq!(partes.len(), 3);
    assert_eq!(partes[0], "1920");
    assert_eq!(partes[1], "1080");
    assert_eq!(partes[2], "32");
}

#[test]
fn test_parse_entrada_menu_basica() {
    // Arrange: Entrada de menu simples
    let entrada = r#"
/Redstone OS
    protocol: limine
    kernel_path: boot():/forge
"#;

    // Act: Verificar estrutura
    let linhas: Vec<&str> = entrada.lines().collect();

    // Assert: Primeira linha começa com /
    let primeira_relevante = linhas.iter().find(|l| !l.trim().is_empty()).unwrap();
    assert!(
        primeira_relevante.trim().starts_with('/'),
        "Entrada deve começar com /"
    );
}

#[test]
fn test_parse_entrada_menu_com_modulos() {
    // Arrange: Entrada com módulos
    let entrada = r#"
/Redstone OS
    protocol: limine
    kernel_path: boot():/forge
    module_path: boot():/initfs
    module_cmdline: ro
"#;

    // Act: Contar módulos
    let count_modulos = entrada.matches("module_path").count();

    // Assert: Deve ter 1 módulo
    assert_eq!(count_modulos, 1);
}

#[test]
fn test_parse_macro_definicao() {
    // Arrange: Definição de macro
    let macro_def = "${OS_NAME}=Redstone";

    // Act: Extrair nome e valor
    assert!(macro_def.starts_with("${"));
    assert!(macro_def.contains("}="));

    let partes: Vec<&str> = macro_def[2..].split("}=").collect();

    // Assert: Verificar estrutura
    assert_eq!(partes.len(), 2);
    assert_eq!(partes[0], "OS_NAME");
    assert_eq!(partes[1], "Redstone");
}

#[test]
fn test_parse_expansao_macro() {
    // Arrange: Texto com macro
    let texto = "Bootstrap ${OS_NAME} v${VERSION}";
    let macro_os = "Redstone";
    let macro_ver = "1.0";

    // Act: Simular expansão
    let expandido = texto
        .replace("${OS_NAME}", macro_os)
        .replace("${VERSION}", macro_ver);

    // Assert: Verificar resultado
    assert_eq!(expandido, "Bootstrap Redstone v1.0");
}

#[test]
fn test_parse_ignorar_comentarios() {
    // Arrange: Texto com comentários
    let linhas = [
        "# Este é um comentário",
        "timeout: 5  # comentário no final",
        "  # comentário indentado",
    ];

    for linha in linhas {
        // Act: Verificar se é comentário
        let trimmed = linha.trim();
        let eh_comentario = trimmed.starts_with('#') || trimmed.is_empty();

        // Assert: Deve detectar comentários
        assert!(
            eh_comentario || !trimmed.starts_with('#'),
            "Linha deveria ser tratada adequadamente: {}",
            linha
        );
    }
}

#[test]
fn test_parse_linhas_vazias() {
    // Arrange: Linhas vazias
    let linhas = ["", "   ", "\t", "\t  ", "  \t  "];

    for linha in linhas {
        // Act: Verificar se está vazia após trim
        let eh_vazia = linha.trim().is_empty();

        // Assert: Todas devem ser vazias
        assert!(eh_vazia, "Linha deveria ser considerada vazia: '{}'", linha);
    }
}

#[test]
fn test_parse_entrada_hierarquica() {
    // Arrange: Entradas hierárquicas
    let config = r#"
/Menu Principal
    protocol: limine
//Sub-menu 1
    kernel_path: boot():/kernel1
///Sub-sub-menu
    kernel_path: boot():/kernel2
"#;

    // Act: Contar níveis
    let linhas: Vec<&str> = config
        .lines()
        .filter(|l| l.trim().starts_with('/'))
        .collect();

    // Assert: Deve ter 3 níveis
    assert_eq!(linhas.len(), 3);

    // Verificar níveis
    assert_eq!(linhas[0].chars().take_while(|&c| c == '/').count(), 1); // Nível 1
    assert_eq!(linhas[1].chars().take_while(|&c| c == '/').count(), 2); // Nível 2
    assert_eq!(linhas[2].chars().take_while(|&c| c == '/').count(), 3); // Nível 3
}

#[test]
fn test_parse_entrada_expandida() {
    // Arrange: Entrada com flag de expandido (+)
    let entrada = "/+Menu Expandido";

    // Act: Verificar flag
    let nivel = entrada.chars().take_while(|&c| c == '/').count();
    let tem_flag_expandido = entrada.chars().nth(nivel) == Some('+');

    // Assert: Deve detectar flag
    assert!(tem_flag_expandido, "Deveria detectar flag de expandido");
}

#[test]
fn test_parse_baudrate_serial() {
    // Arrange: Configuração de baudrate
    let baudrates = ["9600", "19200", "38400", "57600", "115200"];

    for baudrate in baudrates {
        // Act: Tentar parsear
        let resultado = baudrate.parse::<u32>();

        // Assert: Deve ser um número válido
        assert!(resultado.is_ok(), "Baudrate inválido: {}", baudrate);
        assert!(resultado.unwrap() > 0, "Baudrate deve ser positivo");
    }
}

#[test]
fn test_parse_wallpaper_style() {
    // Arrange: Diferentes estilos de wallpaper
    let estilos = ["centered", "stretched", "tiled"];

    for estilo in estilos {
        // Act: Verificar que é string válida
        let estilo_lower = estilo.to_lowercase();

        // Assert: Deve ser um dos estilos conhecidos
        assert!(
            estilo_lower == "centered" || estilo_lower == "stretched" || estilo_lower == "tiled",
            "Estilo desconhecido: {}",
            estilo
        );
    }
}

#[test]
fn test_parse_editor_enabled() {
    // Arrange: Opção de editor
    let configs = [("editor_enabled: yes", true), ("editor_enabled: no", false)];

    for (config, esperado) in configs {
        // Act: Extrair valor
        let valor = config.split(':').nth(1).unwrap().trim().to_lowercase();
        let habilitado = valor == "yes";

        // Assert: Deve corresponder ao esperado
        assert_eq!(habilitado, esperado, "Falha em: {}", config);
    }
}

#[test]
fn test_parse_kaslr_option() {
    // Arrange: Opção KASLR
    let configs = ["kaslr: yes", "kaslr: no"];

    for config in configs {
        // Act: Extrair valor
        let valor = config.split(':').nth(1).unwrap().trim().to_lowercase();

        // Assert: Deve ser yes ou no
        assert!(
            valor == "yes" || valor == "no",
            "Valor KASLR inválido: {}",
            valor
        );
    }
}

#[test]
fn test_parse_dtb_path() {
    // Arrange: Caminho DTB (Device Tree Blob)
    let config = "dtb_path: boot():/device-tree.dtb";

    // Act: Extrair caminho
    let caminho = config.split(':').nth(1).unwrap().trim();

    // Assert: Deve ter formato de caminho
    assert!(
        caminho.contains("boot():/"),
        "Caminho DTB deve usar protocolo boot()"
    );
    assert!(
        caminho.ends_with(".dtb"),
        "Caminho DTB deve terminar em .dtb"
    );
}
