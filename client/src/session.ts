import useWebSocket from "./websocket"
import "./css/base.css"
import "./css/session.css"

import '@fortawesome/fontawesome-free/js/fontawesome'
import '@fortawesome/fontawesome-free/js/solid'
import '@fortawesome/fontawesome-free/js/regular'
import '@fortawesome/fontawesome-free/js/brands'

interface DeviceInfo {
    id: string;
    name: string;
    dev_type: string;
}

interface TrackInfo {
    name: string;
    artists: string[];
    id: string;
}

interface SearchResults {
    tracks: TrackInfo[];
}

interface StateUpdate {
    track: TrackInfo | null;
    queue: TrackInfo[];
}

enum Context {
    Host,
    Peer,
    None
}

let context = Context.None

switch(document.querySelector<HTMLParagraphElement>("#context").innerText) {
    case "host": {
        context = Context.Host
        break
    }
    case "peer": {
        context = Context.Peer
        break
    }
}

const logout = () => {
    window.location.href = "/session/logout"
}

let votedTracksCache: string[] = [];

const onMessageCb = (ev: MessageEvent<any>) => {
    let result = JSON.parse(ev.data)

    switch (result.type) {
        case "Devices": {
            let devices = result.payload as DeviceInfo[]
            populateAndDisplayDevicesNav(devices)
            break
        }
        case "SearchResult": {
            let searchResults = result.payload as SearchResults
            searchResultsList.textContent = ""
            searchResultsList.appendChild(createTrackList(searchResults.tracks, "Add", queueTrack))
            break
        }
        case "StateUpdate": {
            let stateUpdate = result.payload as StateUpdate
            trackQueue.textContent = ""

            if (stateUpdate.track) {
                let currentTrackContainer = document.createElement("div")
                currentTrackContainer.id = "current-track-container"
                let paragraph = document.createElement("p")
                const b = document.createElement("b")
                b.textContent = stateUpdate.track.name + " - " + stateUpdate.track.artists
                paragraph.appendChild(b)
                currentTrackContainer.appendChild(paragraph)

                let volumeIcon = document.createElement("i")
                volumeIcon.classList.add("fa")
                volumeIcon.classList.add("fa-volume-up")
                volumeIcon.ariaHidden = "true"
                volumeIcon.id = "volume-icon"
                currentTrackContainer.appendChild(volumeIcon)

                trackQueue.appendChild(currentTrackContainer)
            }
            
            trackQueue.appendChild(createTrackList(stateUpdate.queue, "Vote", voteTrack))
            
            const votedTracksRequest = { type: "VotedTracks" }
            doSend(JSON.stringify(votedTracksRequest))
            break
        }
        case "Shutdown": {
            logout()
            break
        }
        case "Transfer": {
            let resultCode = result.payload as string

            if (resultCode === "OK") {
                devicesNav.style.width = "0"
            }

            break
        }
        case "VotedTracks": {
            votedTracksCache = result.payload as string[]
            disableVotingForTracks(votedTracksCache)
            break
        }
    }
}

const { doConnect, doSend } = useWebSocket(onMessageCb)

const onOpenCb = () => {
    if (context === Context.Host) {
        const devicesRequest = { type: "Devices" }
        doSend(JSON.stringify(devicesRequest))
    }

    const stateRequest = { type: "State" }
    doSend(JSON.stringify(stateRequest))
}

doConnect(onOpenCb)

const settingsNav = document.querySelector<HTMLDivElement>("#settings-nav")
const settingsNavContent = document.querySelector<HTMLDivElement>("#settings-nav-content")
const devicesNav = document.querySelector<HTMLDivElement>("#devices-nav")
const closeDevicesNavButton = document.querySelector<HTMLButtonElement>("#close-devices-nav-btn")
    closeDevicesNavButton.addEventListener("click", (ev) => {
        devicesNav.style.width = "0"
})


const closeSettingsNavButton = document.querySelector<HTMLButtonElement>("#close-settings-nav-btn")
closeSettingsNavButton.addEventListener("click", (ev) => {
    ev.preventDefault()
    settingsNav.style.width = "0" 
})

if (context === Context.Host) {
    const copyJoinUrlButton = document.createElement("button")
    copyJoinUrlButton.innerText = "Copy URL"
    copyJoinUrlButton.addEventListener("click", (ev) => {
        ev.preventDefault()
        const sessionId = document.querySelector<HTMLButtonElement>("#session_id")
        const { location } = window
        const url = `${location.protocol}//${location.host}/join/${sessionId.innerText}`
        console.log("Copy join url ", url)
        
        navigator.clipboard.writeText(url)
    })
    copyJoinUrlButton.classList.add("nav-btn")
    settingsNavContent.appendChild(copyJoinUrlButton)

    const devicesButton = document.createElement("button")
    devicesButton.innerText = "Devices"
    devicesButton.addEventListener("click", (ev) => {
        ev.preventDefault()
        const devicesRequest = { type: "Devices" }
        doSend(JSON.stringify(devicesRequest))
        settingsNav.style.width = "0"
    })
    devicesButton.classList.add("nav-btn")
    settingsNavContent.appendChild(devicesButton)

    const endSessionButton = document.createElement("button")
    endSessionButton.innerText = "End session"
    endSessionButton.id = "end-session-btn"
    endSessionButton.addEventListener("click", (ev) => {
        ev.preventDefault()
        const killRequest = { type: "Kill"}
        doSend(JSON.stringify(killRequest))
    })
    settingsNavContent.appendChild(endSessionButton)

} else if (context === Context.Peer) {
    const exitSessionButton = document.createElement("button")
    exitSessionButton.innerText = "Exit session"
    exitSessionButton.addEventListener("click", (ev) => {
        ev.preventDefault()
        logout()
    })
    settingsNavContent.appendChild(exitSessionButton)
}

const queueTrack = (ev: MouseEvent, trackId: string) => {
    console.log("Queue track ", trackId)
    const queueRequest = { type: "Queue", uri: trackId }
    doSend(JSON.stringify(queueRequest))
    closeSearchNavButton.click()
}

const voteTrack = (ev: MouseEvent, trackId: string) => {
    console.log("Vote for track ", trackId)
    const voteRequets = { type: "Vote", uri: trackId }
    doSend(JSON.stringify(voteRequets))
}

const createTrackListEntry = (info: TrackInfo, buttonText: string, onClickCb: (ev: MouseEvent, trackId: string) => void) => {

    var listEntry = document.createElement("li")
    listEntry.classList.add("track-container")
    
    var paragraph = document.createElement("p")
    paragraph.textContent = info.name + " - " + info.artists
    listEntry.appendChild(paragraph)

    const callback = (ev: MouseEvent, trackId: string) => {
        ev.preventDefault()
        onClickCb(ev, trackId)
    }
    const swap = function (trackId: string, ev: MouseEvent) {
        return this(ev, trackId);
    }
    var button = document.createElement("button")
    button.innerText = buttonText
    button.addEventListener("click", swap.bind(callback, info.id))
    button.id = info.id

    if (buttonText === "Add") {
        button.classList.add("nav-btn"); //TODO: ugly..
    }


    if (votedTracksCache.includes(info.id)) {
        button.className = "voted-btn"
        button.disabled = true
    }

    listEntry.appendChild(button)

    return listEntry
}

const createTrackList = (tracks: TrackInfo[], buttonText: string, onClickCb: (ev: MouseEvent, trackId: string) => void) => {
    var container = document.createElement("ul")
    
    for (var track of tracks) {
        container.appendChild(createTrackListEntry(track, buttonText, onClickCb))
    }

    return container
}

const searchResultsList = document.querySelector<HTMLDivElement>("#search-results")
const trackQueue = document.querySelector<HTMLDivElement>("#track-queue")
const searchInput = document.querySelector<HTMLInputElement>("#search-input")
const searchNav = document.querySelector<HTMLButtonElement>("#search-nav")

const searchToggle = document.querySelector<HTMLButtonElement>("#search-toggle")
searchToggle.addEventListener("click", (ev) => {
    ev.preventDefault()
    searchNav.style.height = "100%"
})

const searchButton = document.querySelector<HTMLButtonElement>("#search-btn")
searchButton.addEventListener("click", (ev) => {
    ev.preventDefault()

    if (!searchInput.value) return

    const input = searchInput.value
    searchInput.value = ""

    const searchRequest = { type: "Search", query: input }
    doSend(JSON.stringify(searchRequest))
})

const closeSearchNavButton = document.querySelector<HTMLButtonElement>("#close-search-nav-btn")
closeSearchNavButton.addEventListener("click", (ev) => {
    ev.preventDefault()
    searchNav.style.height = "0"
})

const settingsToggle = document.querySelector<HTMLButtonElement>("#settings-toggle")
settingsToggle.addEventListener("click", (ev) => {
    ev.preventDefault()
    const settingsNav = document.querySelector<HTMLButtonElement>("#settings-nav")
    settingsNav.style.width = "100%"
})

const populateAndDisplayDevicesNav = (devices: DeviceInfo[]) => {
    const deviceList = document.querySelector<HTMLDivElement>("#device-list")
    deviceList.innerText = ""

    let p = document.createElement("p")
    let b = document.createElement("b")
    b.innerText = "Select playback device"
    p.appendChild(b)
    deviceList.appendChild(p)

    for (const device of devices) {
        let container = document.createElement("div")
        container.className = "device"

        const p = createDeviceParagraph(device)

        let button = document.createElement("button")
        button.innerText = "+"

        const callback = (ev: MouseEvent, deviceId: string) => {
            ev.preventDefault()
            console.log("Transfer to device with id ", deviceId)
            const transferRequest = { type: "Transfer", device_id: deviceId }
            doSend(JSON.stringify(transferRequest))
        }
        const swap = function (deviceId: string, ev: MouseEvent) {
            return this(ev, deviceId);
        }
        
        button.addEventListener("click", swap.bind(callback, device.id))
        button.classList.add("nav-btn")

        container.appendChild(p)
        container.appendChild(button)
        deviceList.appendChild(container)
    }

    devicesNav.style.width = "100%"
}

const createDeviceParagraph = (device: DeviceInfo) => {
    let p = document.createElement("p")
    const icon = document.createElement("i")
    icon.classList.add("fa")
    icon.ariaHidden = "true"

    if (device.dev_type === "Computer") {
        icon.classList.add("fa-desktop")
    } else if (device.dev_type === "Tablet") {
        icon.classList.add("fa-tablet")
    } else if (device.dev_type === "Smartphone") {
        icon.classList.add("fa-mobile")
    } else if (device.dev_type === "Speaker") {
        icon.classList.add("fa-music")
    } else if (device.dev_type === "Tv") {
        icon.classList.add("fa-television")
    } else {
        icon.classList.add("fa-question")
    }

    p.appendChild(icon)
    p.innerHTML += ("  " + device.name)

    return p
}

const disableVotingForTracks = (tracks: string[]) => {
    const trackList = trackQueue.getElementsByTagName("ul")
    if (trackList && trackList.length > 0) {
        for (const li of Array.from(trackList[0].children)) {
            const button = li.getElementsByTagName("button")[0];
            if (tracks.includes(button.id)) {
                button.disabled = true
                button.className = "voted-btn"
            }
        }
    }
}
