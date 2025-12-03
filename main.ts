import { Application } from "https://deno.land/x/oak@v12.6.1/mod.ts";
import { oakCors } from "https://deno.land/x/cors/mod.ts";
import { load } from "https://deno.land/std@0.201.0/dotenv/mod.ts";
import { Router } from "https://deno.land/x/oak@v12.6.1/mod.ts"
import buzzerRouter from "./src/buzzer.ts";

const app = new Application();
const env = await load(".env");
const router = new Router();

app.use(
  oakCors({
    origin: env.FRONTEND_URL || "http://localhost:5173", 
    methods: ["GET", "POST", "PUT", "DELETE", "OPTIONS"], 
    allowedHeaders: ["Content-Type", "Authorization"], 
    credentials: true,
  }),
);
!
router.get("/get_cookies", (ctx : any) => {
  ctx.response.status = 200;
  ctx.response.body = "Miam les cookies !";
});

app.use(buzzerRouter.routes());
app.use(buzzerRouter.allowedMethods());

app.use(router.routes());
app.use(router.allowedMethods());

const PORT = Number(env.PORT) || 3000;
console.log(`Server is running on port ${PORT}`);
await app.listen({ port: PORT });