/// Hooks into the Roblox authentication flow and extracts credentials to send back to the bootstrapper.

async function getCsrfToken() {
  // URL does not matter
  return fetch("https://auth.roblox.com/v2/login", {
    credentials: "include",
    method: "POST",
  }).then((response) => response.headers.get("x-csrf-token"));
}

async function getAuthTicket() {
  const csrfToken = await getCsrfToken();
  if (!csrfToken) throw "Failed to get CSRF token";

  return fetch("https://auth.roblox.com/v1/authentication-ticket", {
    method: "POST",
    credentials: "include",
    headers: {
      Referer: "https://www.roblox.com",
      "x-csrf-token": csrfToken,
    },
  }).then((response) => response.headers.get("rbx-authentication-ticket"));
}

async function redeemAuthTicket() {
  const authenticationTicket = await getAuthTicket();
  if (!authenticationTicket) throw "Failed to get auth ticket";

  const req = new XMLHttpRequest();
  req.addEventListener("load", () => {
    console.log(req);
    console.log(req.getAllResponseHeaders());
    console.log(req.status, req.statusText)
    console.log(req.responseText)
  });

  req.open("POST", "https://auth.roblox.com/v1/authentication-ticket/redeem");
  req.withCredentials = true;
  req.setRequestHeader("Referer", "https://www.roblox.com");
  req.setRequestHeader("Content-Type", "application/json;charset=UTF-8");
  req.setRequestHeader("rbxauthenticationnegotiation", authenticationTicket);
  req.send(JSON.stringify({ authenticationTicket }));

  // return fetch("https://auth.roblox.com/v1/authentication-ticket/redeem", {
  //     method: "POST",
  //     credentials: "include",
  //     headers: {
  //         Referer: "https://www.roblox.com",
  //         "Content-Type": "text/json",
  //         // "User-Agent": "Roblox/WinInet",
  //         rbxauthenticationnegotiation: authenticationTicket,
  //     },
  //     body: JSON.stringify({
  //         authenticationTicket,
  //     }),
  // })
  //     .then((response) => response.headers.get("Set-Cookie"));
}

async function run() {
  // window.addEventListener("load", async () => {
  // Dark mode is nice :)
//   document.body.classList.remove("light-theme");
//   document.body.classList.add("dark-theme");

  const currentPath = window.location.href;
  console.debug("Auth hook loaded page at URL: " + currentPath);

  if (!currentPath.endsWith("home")) {
    // We only care once the use has progressed from login to home screen
    return;
  }

  console.debug("Page loaded to home screen, extracting credentials");

  const redeemedAuthTicket = await redeemAuthTicket();
  // if (!redeemedAuthTicket) throw "Failed to redeem auth ticket";

  // console.log(redeemAuthTicket)
  // });
}

run();
