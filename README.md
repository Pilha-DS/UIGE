# UIGE — User Interface Generation Engine

Motor para Linux capaz de gerar interfaces gráficas automaticamente para aplicações originalmente operadas por terminal, utilizando descrições declarativas em JSON ou descoberta automática dos comandos da aplicação.

```
                     ┌─────────────────────┐
                     │       UIGE UI       │
                     │ GTK / Qt / WebView  │
                     └──────────┬──────────┘
                                │
                         UI Definition
                                │
                     ┌──────────▼──────────┐
                     │    UIGE Engine      │
                     │                    │
                     │ Components         │
                     │ Validation         │
                     │ Command Builder    │
                     │ Output Parser      │
                     └───────┬─────┬──────┘
                             │     │
              ┌──────────────┘     └──────────────┐
              │                                    │
     ┌────────▼────────┐                 ┌─────────▼─────────┐
     │ Manifest Mode   │                 │  Universal Mode   │
     │                 │                 │                   │
     │ *.uige.json     │                 │ --help            │
     │ criado à mão    │                 │ help              │
     │ ou oficialmente │                 │ man               │
     └────────┬────────┘                 │ completions       │
              │                          └─────────┬─────────┘
              └──────────────┬─────────────────────┘
                             │
                     ┌───────▼────────┐
                     │ Command Runner │
                     │                │
                     │ git status     │
                     │ pacman -S ...  │
                     │ docker ps      │
                     └───────┬────────┘
                             │
                     ┌───────▼────────┐
                     │ Linux CLI App  │
                     └────────────────┘
```

1. Manifest Mode

É o modo confiável.

Existe um JSON dizendo exatamente como determinada aplicação funciona.
