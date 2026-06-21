# OpenHOTAS — Plano V1.21: CDC Serial Debug (Revisão Final)

## Status

* [ ] Não iniciado

## Dependências

* V1.2 compilado sem erros
* Arquitetura V1.2 estabilizada
* Embassy RP 0.10

---

# Objetivo

Adicionar uma interface USB CDC ACM para diagnóstico e telemetria em tempo real utilizando o mesmo cabo USB do HID Gamepad.

O CDC será exclusivamente um canal auxiliar de debug.

Falhas, desconexões ou indisponibilidade da porta COM nunca poderão afetar:

* HID Gamepad
* Pipeline de sensores
* Tasks críticas
* Latência do sistema

---

# Escopo

## Alterados

| Arquivo                 | Alteração                |
| ----------------------- | ------------------------ |
| Cargo.toml              | Adicionar ufmt           |
| src/main.rs             | Criar interface CDC      |
| src/tasks/diagnostic.rs | Implementar saída serial |

## Não Alterados

* spi_bus.rs
* sensors/
* filters/
* axis/
* hid_gamepad.rs
* descriptor.rs
* input_task
* hid_task
* usb_task
* pipeline de sinal
* lógica HID

---

# Arquitetura USB

O dispositivo USB continuará expondo duas funções independentes:

```text
USB Device: OpenHOTAS
├── HID Gamepad
└── CDC ACM (COM Virtual)
```

Windows:

```text
Dispositivos de Jogo
└── OpenHOTAS Gamepad

Portas (COM e LPT)
└── OpenHOTAS CDC
```

Linux:

```text
/dev/input/jsX
/dev/ttyACMX
```

---

# Cargo.toml

```toml
ufmt = "0.2"
```

---

# main.rs

## Imports

```rust
use embassy_usb::class::cdc_acm::{
    CdcAcmClass,
    State as CdcState,
};
```

## Estado Global

```rust
static mut CDC_STATE: CdcState = CdcState::new();
```

## Criação da Interface

Antes de:

```rust
builder.build();
```

Adicionar:

```rust
let mut cdc = CdcAcmClass::new(
    &mut builder,
    unsafe { &mut CDC_STATE },
    64,
);
```

### Justificativa

USB Full Speed suporta:

```text
8
16
32
64
```

64 bytes é o maior packet size válido.

Não utilizar 128.

---

## Split

Após build:

```rust
let (cdc_sender, _cdc_receiver) = cdc.split();
```

---

## Spawn

```rust
spawner
    .spawn(
        tasks::diagnostic::diagnostic_task(
            cdc_sender
        )
    )
    .unwrap();
```

---

# diagnostic.rs

## Imports

```rust
use embassy_rp::peripherals::USB;
use embassy_rp::usb::Driver;

use embassy_usb::class::cdc_acm::Sender;
use embassy_usb::driver::EndpointError;

use embassy_time::Timer;

use core::sync::atomic::Ordering;

use ufmt::uwrite;
```

---

# WriteCursor

O ufmt exige um tipo que implemente uWrite.

```rust
struct WriteCursor<'a> {
    buf: &'a mut [u8],
    pos: usize,
    overflow: bool,
}
```

---

## Construtor

```rust
impl<'a> WriteCursor<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        Self {
            buf,
            pos: 0,
            overflow: false,
        }
    }

    fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.pos]
    }

    fn overflowed(&self) -> bool {
        self.overflow
    }
}
```

---

## Implementação uWrite

```rust
impl ufmt::uWrite for WriteCursor<'_> {
    type Error = ();

    fn write_str(
        &mut self,
        s: &str,
    ) -> Result<(), Self::Error> {

        let bytes = s.as_bytes();

        if self.pos + bytes.len() > self.buf.len() {
            self.overflow = true;
            return Err(());
        }

        self.buf[
            self.pos..
            self.pos + bytes.len()
        ]
        .copy_from_slice(bytes);

        self.pos += bytes.len();

        Ok(())
    }
}
```

---

# Helper de Envio

Objetivo:

* respeitar limite USB
* interromper imediatamente em caso de erro
* evitar envio parcial desnecessário

```rust
async fn send_text(
    cdc: &mut Sender<'static, Driver<'static, USB>>,
    data: &[u8],
) -> Result<(), EndpointError> {

    for chunk in data.chunks(64) {
        cdc.write_packet(chunk).await?;
    }

    Ok(())
}
```

---

# Task Principal

```rust
#[embassy_executor::task]
pub async fn diagnostic_task(
    mut cdc: Sender<'static, Driver<'static, USB>>,
) -> !
{
    loop {

        cdc.wait_connection().await;

        if send_text(
            &mut cdc,
            b"OpenHOTAS CDC Debug Connected\r\n"
        )
        .await
        .is_err()
        {
            continue;
        }

        loop {

            let cycles =
                runtime_stats::SENSOR_CYCLES
                    .load(Ordering::Relaxed);

            let max_us =
                runtime_stats::MAX_CYCLE_US
                    .load(Ordering::Relaxed);

            let last_us =
                runtime_stats::LAST_CYCLE_US
                    .load(Ordering::Relaxed);

            let reports =
                runtime_stats::REPORTS_SENT
                    .load(Ordering::Relaxed);

            let errors =
                runtime_stats::SEND_ERRORS
                    .load(Ordering::Relaxed);

            let mut buf = [0u8; 128];

            let mut w =
                WriteCursor::new(&mut buf);

            let _ = uwrite!(
                w,
                "cycles={}\r\n",
                cycles
            );

            let _ = uwrite!(
                w,
                "last={}\r\n",
                last_us
            );

            let _ = uwrite!(
                w,
                "max={}\r\n",
                max_us
            );

            let _ = uwrite!(
                w,
                "reports={}\r\n",
                reports
            );

            let _ = uwrite!(
                w,
                "errors={}\r\n",
                errors
            );

            if max_us > MAX_INPUT_CYCLE_US {
                let _ = uwrite!(
                    w,
                    "WARN:max_cycle\r\n"
                );
            }

            if send_text(
                &mut cdc,
                w.as_bytes()
            )
            .await
            .is_err()
            {
                break;
            }

            if w.overflowed() {

                if send_text(
                    &mut cdc,
                    b"[diag truncated]\r\n"
                )
                .await
                .is_err()
                {
                    break;
                }
            }

            Timer::after_secs(
                DIAGNOSTIC_INTERVAL_SECS
            )
            .await;
        }
    }
}
```

---

# Comportamento Esperado

Ao abrir a COM:

```text
OpenHOTAS CDC Debug Connected

cycles=12345
last=312
max=487
reports=12340
errors=0
```

Sem esperar 5 segundos pelo primeiro log.

---

# Reconexão Automática

Fluxo:

```text
Terminal aberto
        ↓
wait_connection()
        ↓
Logs ativos
        ↓
Terminal fechado
        ↓
EndpointError
        ↓
break
        ↓
wait_connection()
        ↓
Aguarda nova conexão
```

Sem panic.

Sem reset.

Sem impacto no HID.

---

# Checklist

* [ ] Adicionar ufmt
* [ ] Declarar CDC_STATE
* [ ] Criar CdcAcmClass
* [ ] Packet size 64
* [ ] Implementar WriteCursor
* [ ] Implementar send_text()
* [ ] Fragmentação em chunks de 64 bytes
* [ ] Implementar wait_connection()
* [ ] Implementar banner de conexão
* [ ] Implementar detecção de overflow
* [ ] Validar build release
* [ ] Validar clippy
* [ ] Validar HID
* [ ] Validar CDC
* [ ] Validar abertura/fechamento da COM
* [ ] Validar desconexão física do cabo
* [ ] Teste contínuo de 30 minutos

---

# Critério de Aprovação

A V1.21 será considerada concluída quando:

1. HID continuar operando normalmente.
2. CDC enumerar corretamente.
3. Primeiro log aparecer imediatamente após abrir a COM.
4. Logs forem emitidos a cada 5 segundos.
5. Fechar o terminal não causar falha.
6. Reabrir o terminal restaurar os logs.
7. Remover e reconectar o cabo USB restaurar os logs.
8. Nenhum panic ocorrer durante testes prolongados.
9. Nenhum módulo crítico do OpenHOTAS precisar ser alterado.
