import "./css/base.css"
import "./css/index.css"
import axios from 'axios'

const createSessionButton = document.querySelector<HTMLButtonElement>("#create-btn")
createSessionButton.addEventListener("click", async (ev) => {
    ev.preventDefault()

    try {
        let result = await axios.get("/create"); 
        window.location.href = result.data;
    } catch (error) {
    }
})