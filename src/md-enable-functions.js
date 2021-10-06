export function enableRipple(querySelector) {
    let elements = document.querySelectorAll(querySelector);
    for (let i = 0; i < elements.length; i++) {
        let element = elements[i];
        mdc.ripple.MDCRipple.attachTo(element);    
    }    
}

export function enableTextField(querySelector) {
    let elements = document.querySelectorAll(querySelector);
    for (let i = 0; i < elements.length; i++) {
        let element = elements[i];
        new mdc.textField.MDCTextField(element);
    }    
}