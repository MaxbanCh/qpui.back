import { Router } from "https://deno.land/x/oak@v12.6.1/mod.ts"

const router = new Router();

interface WebSocketWithData extends WebSocket {
  data?: {
    userId: string;
    username: string;
    roomCode?: string;
  };
}

// Types spécifiques aux salles de buzzer
interface BuzzerRoom {
  code: string;
  host: string;
  players: { id: string; username: string; status?: string; locked?: boolean }[];
  scores: Record<string, number>;
  activeBuzzer: string | null;
}

// Collections pour stocker les salles et les connexions
export const buzzerRooms = new Map<string, BuzzerRoom>();
export const buzzerConnections: WebSocketWithData[] = [];

// Fonction pour générer un code de salle aléatoire
function generateRoomCode(): string {
  return Math.random().toString(36).substring(2, 8).toUpperCase();
}

// Fonctions utiles pour la communication
function broadcastToRoom(roomCode: string, data: any) {
  const room = buzzerRooms.get(roomCode);
  if (!room) return;

  buzzerConnections.forEach((client) => {
    if (client.data?.roomCode === roomCode && client.readyState === 1) {
      client.send(JSON.stringify(data));
    }
  });
}

// Handlers pour les différentes actions
function createBuzzerRoom(data: any, ws: WebSocketWithData) {
  const roomCode = generateRoomCode();
  const newRoom: BuzzerRoom = {
    code: roomCode,
    host: data.userId,
    players: [{ id: data.userId, username: data.username }],
    scores: {},
    activeBuzzer: null
  };
  
  buzzerRooms.set(roomCode, newRoom);
  ws.data = { userId: data.userId, username: data.username, roomCode };
  ws.send(JSON.stringify({ type: "ROOM_CREATED", room: newRoom }));
}

function joinBuzzerRoom(data: any, ws: WebSocketWithData) {
  const room = buzzerRooms.get(data.roomCode);
  if (room) {
    // Vérifier si le joueur est déjà dans la salle
    if (!room.players.some(p => p.id === data.userId)) {
      room.players.push({
        id: data.userId,
        username: data.username
      });
    }
    
    ws.data = { userId: data.userId, username: data.username, roomCode: data.roomCode };
    ws.send(JSON.stringify({ type: "ROOM_JOINED", room: room }));

    broadcastToRoom(data.roomCode, {
      type: "PLAYER_JOINED",
      players: room.players
    });
  } else {
    createBuzzerRoom(data, ws);
    // ws.send(JSON.stringify({ 
    //   type: "ERROR", 
    //   message: "Salle introuvable" 
    // }));
  }
}

function pressBuzzer(data: any, ws: WebSocketWithData) {
  const room = buzzerRooms.get(data.roomCode);
  if (!room) return;
  
  const player = room.players.find(p => p.id === data.userId);
  
  // Vérifier si le joueur est verrouillé
  if (player?.locked) {
    ws.send(JSON.stringify({ 
      type: "BUZZER_LOCK",
      locked: true,
      message: "Votre buzzer est verrouillé par l'hôte" 
    }));
    return;
  }
  
  // Si personne n'a encore le buzzer, l'attribuer à ce joueur
  if (room.activeBuzzer === null) {
    room.activeBuzzer = data.userId;
    broadcastToRoom(data.roomCode, {
      type: "BUZZER_PRESSED",
      playerId: data.userId,
      username: data.username,
      timestamp: data.timestamp
    });
  }
}

function resetBuzzer(data: any, ws: WebSocketWithData) {
  const room = buzzerRooms.get(data.roomCode);
  if (!room || room.host !== ws.data?.userId) return;
  
  room.activeBuzzer = null;
  broadcastToRoom(data.roomCode, {
    type: "BUZZER_RESET"
  });
}

function lockPlayerBuzzer(data: any, ws: WebSocketWithData) {
  const room = buzzerRooms.get(data.roomCode);
  if (!room || room.host !== ws.data?.userId) return;
  
  const player = room.players.find(p => p.id === data.playerId);
  if (player) {
    player.locked = data.lock;
    
    broadcastToRoom(data.roomCode, {
      type: "PLAYER_BUZZER_LOCKED",
      playerId: data.playerId,
      locked: data.lock
    });
  }
}

function awardPoints(data: any, ws: WebSocketWithData) {
  const room = buzzerRooms.get(data.roomCode);
  if (!room || room.host !== ws.data?.userId) return;
  
  if (!room.scores[data.playerId]) {
    room.scores[data.playerId] = 0;
  }
  
  room.scores[data.playerId] += data.points;
  
  // Mettre à jour les joueurs avec les scores
  const updatedPlayers = room.players.map(player => ({
    ...player,
    score: room.scores[player.id] || 0
  }));
  
  broadcastToRoom(data.roomCode, {
    type: "POINTS_UPDATED",
    scores: room.scores,
    players: updatedPlayers
  });
}

// Endpoint WebSocket
router.get("/BuzzerRoom", (ctx : any) => {
  if (!ctx.isUpgradable) {
    ctx.throw(501);
  }
  console.log("Oh une connexion")
  
  const ws = ctx.upgrade() as WebSocketWithData;
  buzzerConnections.push(ws);

  ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    console.log("Buzzer Room received message:", data.type);

    switch (data.type) {
      case "CREATE_BUZZER_ROOM":
        createBuzzerRoom(data, ws);
        break;
      case "JOIN_BUZZER_ROOM":
        joinBuzzerRoom(data, ws);
        break;
      case "PRESS_BUZZER":
        pressBuzzer(data, ws);
        break;
      case "RESET_BUZZER":
        resetBuzzer(data, ws);
        break;
      case "LOCK_PLAYER_BUZZER":
        lockPlayerBuzzer(data, ws);
        break;
      case "AWARD_POINTS":
        awardPoints(data, ws);
        break;
      case "LEAVE_BUZZER_ROOM":
        // Handled by onclose
        break;
      default:
        console.log("Unknown message type:", data.type);
        break;
    }
  };

  ws.onclose = () => {
    const index = buzzerConnections.indexOf(ws);
    if (index !== -1) {
      if (ws.data?.roomCode) {
        const room = buzzerRooms.get(ws.data.roomCode);
        if (room) {
          room.players = room.players.filter((p) => p.id !== ws.data?.userId);
          
          if (room.players.length === 0) {
            buzzerRooms.delete(ws.data.roomCode);
          } else {
            if (room.host === ws.data.userId) {
              room.host = room.players[0].id;
            }
            
            broadcastToRoom(ws.data.roomCode, {
              type: "PLAYER_LEFT",
              players: room.players,
              newHost: room.host
            });
          }
        }
      }
      buzzerConnections.splice(index, 1);
    }
  };
});

export default router;