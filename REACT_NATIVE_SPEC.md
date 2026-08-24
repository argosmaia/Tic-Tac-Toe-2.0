# 🎮 Especificação Técnica: Velha 2.0 (Ultimate Tic-Tac-Toe) em React Native

Esta documentação analisa minuciosamente a arquitetura do projeto **Velha 2.0 (Ultimate Tic-Tac-Toe)** escrito em Rust e reinterpreta completamente o sistema para a construção de um aplicativo móvel de alta performance utilizando **React Native**, **TypeScript**, **Zustand**, **Tailwind CSS (NativeWind)**, **DaisyUI (tokens para mobile)**, **React Icons (Lucide Icons)** e **Skia/Reanimated** para a interface de jogo.

---

## 📐 1. Documentação do Sistema Original (Rust / egui)

### 1.1. Visão Geral & Regras do Ultimate Tic-Tac-Toe
O Ultimate Tic-Tac-Toe altera drasticamente a dinâmica da Velha tradicional:
1. **Grade 9×9**: O tabuleiro principal é composto por **9 mini-tabuleiros (quadrantes)** organizados em uma grade 3×3. Cada mini-tabuleiro possui 9 células (total de 81 células).
2. **Regra de Amarração (Active Quad)**: Onde o Jogador A joga em um mini-tabuleiro (ex: célula 4 = centro) **força** o Jogador B a jogar no mini-tabuleiro correspondente a essa posição (ex: quadrante 4 = centro).
3. **Livre Escolha**: Se o quadrante de destino já estiver **resolvido** (vencido por X, por O ou empatado), o próximo jogador pode jogar em **qualquer mini-tabuleiro aberto**.
4. **Condição de Vitória Macro**: Para vencer o jogo completo, o jogador deve dominar **3 mini-tabuleiros em linha** no tabuleiro macro (linhas, colunas ou diagonais).

### 1.2. Mapeamento de Domínio e Estado Puro
O sistema original separa o domínio em:
- **`Player`**: `X` | `O`
- **`Cell`**: `Empty` | `Taken(Player)`
- **`QuadState`**: `Open` | `Won(Player)` | `Draw`
- **`GameResult`**: `Winner(Player)` | `Draw`
- **`Board`**: Contém matriz `cells[9][9]`, estados dos quadrantes `quad_states[9]`, `active_quad: Option<usize>`, `current_player: Player` e `result: Option<GameResult>`.

### 1.3. Níveis de Dificuldade da IA
1. **`Noob`**: 80% aleatório + 20% Minimax profundidade 1.
2. **`Jogadora`**: Heurística de passo único (ganhar se puder, bloquear se necessário) + Minimax profundidade 1.
3. **`Master`**: Minimax com Poda Alpha-Beta até profundidade 4 + Heurística local.
4. **`Killer`**: Minimax com Poda Alpha-Beta até profundidade 6 + Heurística macro+micro combinada.
5. **`TheExperience`**: Minimax profundidade 9 + Poda Alpha-Beta com ordenação e desempate guiado por **Mapa de Calor (Heatmap)** estatístico extraído do histórico de jogadas do humano.

### 1.4. Persistência & Histórico (SQLite)
- Armazena perfis de usuário (`profiles`), histórico de partidas (`matches`) e **cada jogada individual por turno** (`match_moves`).
- O heatmap é gerado contando a frequência com que o humano jogou em cada par `(quadrante, célula)` nas suas partidas passadas.

### 1.5. Comunicação P2P (iroh)
- Utiliza a biblioteca `iroh` (QUIC/DERP) em Rust para salas virtuais (Ticket P2P), realizando handshake, envio de jogadas e sinalização de recomeço/desconexão.

---

## 📱 2. Reinterpretação para React Native Mobile

### 2.1. Arquitetura Tecnológica Recomendada

| Camada | Tecnologia no Rust | Substitutos no React Native | Motivo & Benefício no Mobile |
| :--- | :--- | :--- | :--- |
| **Framework Base** | Rust / egui Native | **React Native + Expo** | Facilidade de compilação, performance nativa e suporte a Expo Go. |
| **Linguagem** | Rust 1.70+ | **TypeScript 5.x** | Tipagem estrita equivalente para manter a segurança do domínio. |
| **Gerenciador de Estado**| `App` struct / Mutex | **Zustand** | Leve, performático, zero boilerplate, sem re-renders desnecessários. |
| **Estilização & UI** | `egui` Custom Theme | **Tailwind CSS (NativeWind v4)** | Estilização por classes utilitárias rápida e responsiva. |
| **Design Tokens** | Visual Neon `egui` | **DaisyUI (Adaptado em RN)** | Estilização de botões (`btn`), cards (`card`), modais e badges com temas dark/neon. |
| **Ícones** | Emojis (💀, 🧠) | **Lucide React Native** | Ícones vetoriais profissionais (`Skull`, `Brain`, `Bot`, `Zap`, `X`, `Circle`). |
| **Renderização do Jogo** | `egui::Ui` Painter | **React Native Skia / Reanimated** | Gráficos 60/120 FPS para o tabuleiro 9x9, linhas de vitória e bordas animadas. |
| **Comunicação P2P** | `iroh` (QUIC/DERP) | **WebRTC + PeerJS / Socket.io** | Protocolo P2P web/mobile nativo sem dependências C++ complexas. |
| **Banco de Dados** | `rusqlite` | **`expo-sqlite` / Async Storage** | Banco SQLite nativo rápido para armazenar perfis e jogadas. |

---

## 💡 3. Por Que Usar Expo em PCs com Pouca RAM (4GB a 8GB)?

Desenvolver aplicativos móveis em máquinas com pouca memória RAM (4GB a 8GB) usando o Android Studio ou o emulador tradicional costuma travar o computador. O **Expo Managed Workflow** resolve esse problema:

1. **Zero Emulador Local**: Você instala o app **Expo Go** no seu celular físico (Android ou iOS).
2. **Servidor HTTP Leve**: O servidor de desenvolvimento do Metro Bundler roda no PC consumindo menos de 200MB de RAM.
3. **Live Reload por QR Code**: O celular se conecta ao PC via Wi-Fi ou cabo USB.
4. **Execução via ADB USB (Sem Wi-Fi)**: Com o comando `adb reverse tcp:8081 tcp:8081`, o app roda direto no celular com latência zero e gasto mínimo de memória.
5. **EAS Build na Nuvem**: A geração dos arquivos `.apk` ou `.aab` para instalação direta é feita nos servidores da Expo na nuvem, sem exigir compilação local (Gradle/Java).

---

## 🏗️ 4. Modelo de Dados e Tipagem em TypeScript (`types.ts`)

```typescript
// types/game.ts

export type Player = 'X' | 'O';

export type CellState = null | Player;

export type QuadState = 
  | { type: 'Open' }
  | { type: 'Won'; winner: Player }
  | { type: 'Draw' };

export type GameResult = 
  | { type: 'Winner'; winner: Player }
  | { type: 'Draw' };

export type GameMode = 'Local' | 'VsCpu' | 'P2P';

export type AiLevel = 'Noob' | 'Jogadora' | 'Master' | 'Killer' | 'TheExperience';

export interface Move {
  quad: number; // 0..8
  cell: number; // 0..8
  player: Player;
}

export interface BoardState {
  cells: CellState[][]; // Matrix 9x9 [quadrante][célula]
  quadStates: QuadState[]; // 9 quadrantes macro
  activeQuad: number | null; // null = livre escolha
  currentPlayer: Player;
  result: GameResult | null;
  moveHistory: Move[];
}
```

---

## ⚙️ 5. Gerenciamento de Estado com Zustand (`useGameStore.ts`)

O **Zustand** fornece controle previsível e performático da máquina de estados do tabuleiro.

```typescript
// store/useGameStore.ts
import { create } from 'zustand';
import { BoardState, CellState, GameMode, AiLevel, GameResult, QuadState, Player } from '../types/game';
import { evaluateQuad, checkGameResult, getValidMoves } from '../game/rules';

interface GameStore extends BoardState {
  mode: GameMode;
  aiLevel: AiLevel;
  playerRole: Player; // No P2P: 'X' (Host) ou 'O' (Guest)
  
  // Ações
  resetGame: () => void;
  setGameMode: (mode: GameMode, level?: AiLevel) => void;
  makeMove: (quad: number, cell: number) => boolean;
}

const initialBoard = (): BoardState => ({
  cells: Array.from({ length: 9 }, () => Array(9).fill(null)),
  quadStates: Array(9).fill({ type: 'Open' }),
  activeQuad: null,
  currentPlayer: 'X',
  result: null,
  moveHistory: [],
});

export const useGameStore = create<GameStore>((set, get) => ({
  ...initialBoard(),
  mode: 'Local',
  aiLevel: 'Jogadora',
  playerRole: 'X',

  resetGame: () => set({ ...initialBoard() }),

  setGameMode: (mode, aiLevel = 'Jogadora') => set({ mode, aiLevel }),

  makeMove: (quad, cell) => {
    const state = get();
    if (state.result) return false;

    // Validação
    const validMoves = getValidMoves(state);
    const isValid = validMoves.some(([q, c]) => q === quad && c === cell);
    if (!isValid) return false;

    // Aplica jogada
    const newCells = state.cells.map((qArr, qIdx) =>
      qIdx === quad ? qArr.map((cVal, cIdx) => (cIdx === cell ? state.currentPlayer : cVal)) : qArr
    );

    // Avalia o quadrante afetado
    const newQuadState = evaluateQuad(newCells[quad]);
    const newQuadStates = [...state.quadStates];
    newQuadStates[quad] = newQuadState;

    // Cria estado temporário para checar resultado global
    const tempBoard: BoardState = {
      ...state,
      cells: newCells,
      quadStates: newQuadStates,
    };

    const gameResult = checkGameResult(tempBoard);

    // Determina o próximo quadrante ativo
    const nextQuadState = newQuadStates[cell];
    const nextActiveQuad = nextQuadState.type === 'Open' ? cell : null;

    const nextPlayer: Player = state.currentPlayer === 'X' ? 'O' : 'X';

    set({
      cells: newCells,
      quadStates: newQuadStates,
      activeQuad: gameResult ? null : nextActiveQuad,
      currentPlayer: nextPlayer,
      result: gameResult,
      moveHistory: [...state.moveHistory, { quad, cell, player: state.currentPlayer }],
    });

    return true;
  },
}));
```

---

## 🧠 6. Motor de IA em React Native (Evitando Lags na UI)

No mobile, o algoritmo Minimax em profundidade 4 ou 6 (níveis `Master`, `Killer` e `TheExperience`) pode bloquear a **UI Thread** (JavaScript Main Thread) por alguns milissegundos.

### Solução de Arquitetura no Mobile:
1. **`react-native-worklets-core` ou Web Workers / Async Processing**: A computação da IA é executada de forma assíncrona com `setTimeout(..., 16)` ou em uma Thread secundária de Worklet.
2. **Algoritmo Minimax com Poda Alpha-Beta (Portado para TS)**:

```typescript
// ai/minimax.ts
import { BoardState, Player } from '../types/game';
import { getValidMoves, evaluateQuad, checkGameResult } from '../game/rules';

export function bestMoveAtDepth(board: BoardState, depth: number): [number, number] | null {
  const moves = getValidMoves(board);
  if (moves.length === 0) return null;

  let bestMove = moves[0];
  let bestScore = board.currentPlayer === 'X' ? -Infinity : Infinity;

  for (const [quad, cell] of moves) {
    const nextBoard = simulateMove(board, quad, cell);
    const score = minimax(nextBoard, depth - 1, -Infinity, Infinity, board.currentPlayer !== 'X');
    
    if (board.currentPlayer === 'X') {
      if (score > bestScore) {
        bestScore = score;
        bestMove = [quad, cell];
      }
    } else {
      if (score < bestScore) {
        bestScore = score;
        bestMove = [quad, cell];
      }
    }
  }

  return bestMove;
}
```

---

## 📡 7. Comunicação e Rede P2P Mobile (WebRTC / PeerJS)

Para substituir o módulo Rust `iroh` sem depender de compilação C/C++, a arquitetura recomendada para mobile usa **WebRTC via PeerJS** (ou Socket.io como fallback):

### Fluxo de Conexão:
1. **Host (Criar Sala)**: Gera um ID de Sala aleatório (ex: `VELHA-8X92`) ou QR Code.
2. **Guest (Entrar na Sala)**: Digita o código ou escaneia o QR Code usando a câmera do celular (`expo-camera`).
3. **DataChannel Direto P2P**: A troca de mensagens (jogadas, mensagens de chat, reraise) ocorre com baixíssima latência diretamente entre os dois celulares.

```typescript
// network/protocol.ts
export type NetworkMessage =
  | { type: 'HANDSHAKE'; name: string; avatar: string }
  | { type: 'MOVE'; quad: number; cell: number }
  | { type: 'RESTART_REQUEST' }
  | { type: 'RESTART_RESPONSE'; accept: boolean }
  | { type: 'PING' }
  | { type: 'PONG' };
```

---

## 💾 8. Banco de Dados Local & Persistência (`expo-sqlite`)

No mobile, usamos **`expo-sqlite`** para substituir o `rusqlite`.

```typescript
// storage/db.ts
import * as SQLite from 'expo-sqlite';

const db = SQLite.openDatabaseSync('velha2.db');

export function initDatabase() {
  db.execSync(`
    PRAGMA foreign_keys = ON;

    CREATE TABLE IF NOT EXISTS profiles (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      name TEXT UNIQUE NOT NULL,
      avatar TEXT NOT NULL DEFAULT 'user',
      created_at INTEGER NOT NULL
    );

    CREATE TABLE IF NOT EXISTS matches (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      player_x TEXT NOT NULL,
      player_o TEXT NOT NULL,
      mode TEXT NOT NULL,
      result TEXT NOT NULL,
      duration_s INTEGER,
      played_at INTEGER NOT NULL
    );

    CREATE TABLE IF NOT EXISTS match_moves (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      match_id INTEGER NOT NULL,
      turn INTEGER NOT NULL,
      player TEXT NOT NULL,
      quad INTEGER NOT NULL,
      cell INTEGER NOT NULL,
      FOREIGN KEY (match_id) REFERENCES matches(id) ON DELETE CASCADE
    );
  `);
}
```

### Mapa de Calor (Heatmap) para o nível "The Experience":
Uma query SQL calcula a densidade de jogadas do jogador humano por quadrante e célula `(quad, cell)` normalizada de `0.0` a `1.0`, gerando a matriz `heatmap[9][9]` para a IA.

---

## 🎨 9. Design System Mobile, DaisyUI & Substituição de Emojis por Lucide Icons

### 9.1. Mapeamento de Ícones (Lucide React Native)

Em vez de caracteres unicode de emoji (ex: 💀, 🧠), utilizamos a biblioteca **`lucide-react-native`**:

| Elemento / Conceito | Emoji Antigo | Ícone React (`lucide-react-native`) |
| :--- | :--- | :--- |
| **Jogador X** | `❌` | `<X size={32} color="#00F0FF" />` (Cyan Neon) |
| **Jogador O** | `⭕` | `<Circle size={32} color="#FF0055" />` (Pink Neon) |
| **Nível Noob** | `🌱` | `<Zap size={20} color="#10B981" />` |
| **Nível Jogadora** | `🛡️` | `<Shield size={20} color="#3B82F6" />` |
| **Nível Master** | `🤖` | `<Cpu size={20} color="#8B5CF6" />` |
| **Nível Killer** | `💀` | `<Skull size={20} color="#EF4444" />` |
| **Nível The Experience** | `🧠` | `<Brain size={20} color="#EC4899" />` |
| **Multiplayer Local** | `👥` | `<Users size={24} />` |
| **Partida P2P / Online**| `🌐` | `<Globe size={24} />` |
| **Histórico** | `📜` | `<History size={24} />` |
| **Perfil** | `👤` | `<User size={24} />` |
| **Troféu / Vitória** | `🏆` | `<Trophy size={28} color="#EAB308" />` |

### 9.2. Estilização com DaisyUI Adaptado para NativeWind
DaisyUI fornece classes como `btn`, `btn-primary`, `card`, `badge`, `modal`. No React Native com NativeWind, criamos componentes wrappers reutilizáveis:

```tsx
// components/ui/Button.tsx
import React from 'react';
import { TouchableOpacity, Text } from 'react-native';

interface ButtonProps {
  title: string;
  onPress: () => void;
  variant?: 'primary' | 'secondary' | 'accent' | 'ghost';
  icon?: React.ReactNode;
}

export const Button = ({ title, onPress, variant = 'primary', icon }: ButtonProps) => {
  const variantStyles = {
    primary: 'bg-cyan-500 border-cyan-400 text-black shadow-cyan-500/50',
    secondary: 'bg-pink-600 border-pink-500 text-white shadow-pink-600/50',
    accent: 'bg-purple-600 border-purple-500 text-white shadow-purple-600/50',
    ghost: 'bg-slate-800/80 border-slate-700 text-slate-300',
  };

  return (
    <TouchableOpacity
      onPress={onPress}
      activeOpacity={0.8}
      className={`flex-row items-center justify-center px-6 py-4 rounded-xl border shadow-lg my-2 ${variantStyles[variant]}`}
    >
      {icon && <Text className="mr-2">{icon}</Text>}
      <Text className="font-bold text-base tracking-wide uppercase">{title}</Text>
    </TouchableOpacity>
  );
};
```

---

## 🎨 10. Design das Telas Mobile (`screens/`)

### 10.1. Tela Inicial (`MainMenuScreen.tsx`)
- Header com logo "VELHA 2.0" reluzente em Neon cyan/pink.
- Botões de seleção com ícones Lucide:
  - `<Button title="vs CPU" icon={<Bot />} />`
  - `<Button title="Passa e Joga (Local)" icon={<Users />} />`
  - `<Button title="Partida Online (P2P)" icon={<Globe />} />`
- Footer com ícones rápidos para `<History />`, `<User />` e `<Settings />`.

### 10.2. Tela do Jogo (`GameScreen.tsx` + Skia/Reanimated)
- **Header Status Bar**: Exibe de quem é a vez (`<X />` ou `<Circle />`), contador de tempo e estado da IA.
- **Destaque do Quadrante Ativo (`activeQuad`)**:
  - Se `activeQuad` for um número (ex: 4), aquele mini-tabuleiro recebe uma borda brilhante animada em **pulsar cyan/pink** (usando `react-native-reanimated`).
  - Se `activeQuad === null` (Livre Escolha), todos os mini-tabuleiros abertos brilham suavemente.
- **Quadrante Resolvido**:
  - Quando um jogador vence um mini-tabuleiro, uma marcação gigante semi-transparente do ícone (`<X size={80} />` ou `<Circle size={80} />`) sobrepõe o mini-tabuleiro com uma animação de fade-in e haptic feedback (`expo-haptics`).

---

## 🛠️ 11. Guia Passo a Passo de Implementação e Execução

### Passo 1: Inicializar o Projeto Expo (Sem pesar a RAM)
```bash
# Criação do app com template TypeScript
npx create-expo-app@latest TicTacToeMobile --template blank-typescript
cd TicTacToeMobile

# Instalação das dependências essenciais
npx expo install nativewind tailwindcss react-native-reanimated react-native-gesture-handler lucide-react-native expo-haptics expo-av expo-sqlite zustand
```

### Passo 2: Configurar o NativeWind (Tailwind CSS)
Configurar o `tailwind.config.js` e `babel.config.js` com o plugin do NativeWind para habilitar estilização rápida por classes no React Native.

### Passo 3: Execução e Teste no Celular (Sem Usar Emulador/RAM do PC)

#### Opção A: Conexão via Wi-Fi (Expo Go)
1. Instale o app **Expo Go** na Google Play Store ou App Store no celular.
2. No terminal do PC, rode:
   ```bash
   npx expo start
   ```
3. Abra a câmera do celular e escaneie o **QR Code** exibido no terminal. O app abrirá instantaneamente no celular!

#### Opção B: Conexão Ultra-Rápida via Cabo USB (ADB)
Se o Wi-Fi estiver lento ou oscilando, conecte o celular via cabo USB com a **Depuração USB** ativada:
```bash
# Redireciona a porta do servidor Metro para o celular
adb reverse tcp:8081 tcp:8081

# Inicia o Expo no modo localhost
npx expo start --localhost
```

#### Opção C: Gerar APK Próprio sem Compilar Localmente (EAS Build)
Para ter o aplicativo `.apk` instalado permanentemente no celular sem precisar do PC ligado:
```bash
# Instalar a CLI do EAS
npm install -g eas-cli

# Login na conta gratuita da Expo
eas login

# Iniciar build do APK na nuvem (zero uso de RAM no seu PC)
eas build -p android --profile preview
```
Após alguns minutos, a Expo fornece um link para você baixar o `.apk` direto no celular!

---

## 🎯 12. Resumo da Arquitetura Reinterpretada

1. **Domínio Seguro**: Tipado rigorosamente com TypeScript (`types/game.ts`).
2. **UI Vibrante**: Componentes reutilizáveis inspirados no DaisyUI com NativeWind + Lucide Icons substituindo os emojis.
3. **Performance Visual**: Tabuleiro 9x9 Skia/Reanimated com haptics e animações suaves.
4. **IA Inteligente**: Minimax com Alpha-Beta e mapa de calor estatístico rodando em processamento assíncrono para garantir 60 FPS fluídos.
5. **Desenvolvimento Leve**: 100% compatível com PCs de baixa RAM através do ecossistema Expo + ADB.
