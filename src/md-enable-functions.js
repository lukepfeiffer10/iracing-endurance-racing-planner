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

export function enableTabBar(querySelector) {
    let elements = document.querySelectorAll(querySelector);
    for (let i = 0; i < elements.length; i++) {
        let element = elements[i];
        new mdc.tabBar.MDCTabBar(element);
    }
}

export function enableIconButton(querySelector) {
    let elements = document.querySelectorAll(querySelector);
    for (let i = 0; i < elements.length; i++) {
        let element = elements[i];
        let iconButtonRipple = new mdc.ripple.MDCRipple(element);
        iconButtonRipple.unbounded = true;
    }
}

export function enableSelects(querySelector) {
    let elements = document.querySelectorAll(querySelector);
    for (let i = 0; i < elements.length; i++) {
        let element = elements[i];
        new mdc.select.MDCSelect(element);
    }
}