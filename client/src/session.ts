import "./css/base.css"

const btn = document.querySelector<HTMLButtonElement>("#session-btn")
btn.addEventListener("click", (ev) => {
    ev.preventDefault()
    console.log("session-btn clicked!")
})