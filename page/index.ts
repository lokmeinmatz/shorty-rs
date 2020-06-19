const validityErrorMessages = {
    tooShort: 'URL too short (at least 3 characters)',
    allreadyUsed: 'This URL is allready in use 😒',
    unacceptableChars: 'Only use the chars a-z, A-Z, 0-9, -, _'
}

const BASE_SERVICE_URL = location.href
console.log(`Baseurl: ${BASE_SERVICE_URL}`)

window.onload = () => {
    console.log('TS for shortyRS loaded.')

    // set custom short url prefix display
    const prefixDisplay = document.getElementById('url-prefix')
    prefixDisplay.textContent = BASE_SERVICE_URL

    
    // --- long url ---

    const longUrlInput: HTMLInputElement = <HTMLInputElement> document.getElementById('long-url')
    const longValidityInfo: HTMLParagraphElement = <HTMLParagraphElement> document.getElementById('long-validity-message')

    async function validateLong() {
        const url = longUrlInput.value
        if (url.length == 0) {
            longUrlInput.classList.remove('valid', 'invalid')
            longValidityInfo.style.display = 'none'
            return
        }
        try {
            const res = await fetch(`${BASE_SERVICE_URL}free?long=${url}`)
            if (res.status != 200) {
                console.log('valid long url')
                longUrlInput.classList.add('invalid')
                longUrlInput.classList.remove('valid')
                longValidityInfo.style.display = 'block'
                longValidityInfo.textContent = await res.text()
            }
            else {
                longUrlInput.classList.add('valid')
                longUrlInput.classList.remove('invalid')
                longValidityInfo.style.display = 'none'
            }
        } catch (error) {
            console.error(error)
        }
    }

    longUrlInput.oninput = validateLong

    // --- custom short url ---
    
    const useCustomUrlButton: HTMLInputElement = <HTMLInputElement> document.getElementById('provide-short-url')
    const customUrlInput: HTMLInputElement = <HTMLInputElement> document.getElementById('short-url')
    const shortValidityInfo: HTMLParagraphElement = <HTMLParagraphElement> document.getElementById('short-validity-message')

    shortValidityInfo.style.display = 'none'

    async function validateShort() {
        const url = customUrlInput.value
        if (url.length < 3) {
            shortValidityInfo.style.display = 'block'
            shortValidityInfo.textContent = validityErrorMessages.tooShort
            customUrlInput.classList.remove('valid')
            customUrlInput.classList.add('invalid')
            return
        }

        if (!/^[\w|\d|\-|_]*$/g.test(url)) {
            shortValidityInfo.style.display = 'block'
            shortValidityInfo.textContent = validityErrorMessages.unacceptableChars
            customUrlInput.classList.remove('valid')
            customUrlInput.classList.add('invalid')
            return
        }

        let res = await fetch(`/free?short=${url}`)
        if (!res.ok) {
            shortValidityInfo.style.display = 'block'
            shortValidityInfo.textContent = validityErrorMessages.allreadyUsed
            customUrlInput.classList.remove('valid')
            customUrlInput.classList.add('invalid')
            return
        }

        shortValidityInfo.style.display = 'none'
        customUrlInput.classList.add('valid')
        customUrlInput.classList.remove('invalid')
    }

    customUrlInput.oninput = validateShort
    
    useCustomUrlButton.onchange = evt => {
        const use = useCustomUrlButton.checked
        customUrlInput.disabled = !use
        if(use) {
            // validate
            validateShort()
        }
        else {
            customUrlInput.classList.remove('valid', 'invalid')
        }       
    }
}