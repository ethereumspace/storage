import { storage } from "../../declarations/storage";

document.getElementById("clickMeBtn").addEventListener("click", async () => {
  const name = document.getElementById("name").value.toString();
  // Interact with storage actor, calling the greet method
  const greeting = await storage.greet(name);

  document.getElementById("greeting").innerText = greeting;
});
