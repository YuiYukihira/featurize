

const socket = new WebSocket("ws://localhost:8998");

function reload() {
    const sleep = new Promise(resolve => setTimeout(resolve, 1000));
    sleep.then(() => {
        location.reload();
    })
}

socket.addEventListener("message", (event) => {
    console.log("msg received, reloading");
    reload();
})

socket.addEventListener("error", (event) => {
    reload();
})
