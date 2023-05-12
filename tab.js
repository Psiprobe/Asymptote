var width;
var height;

var command_list;

var command_receive_buffer;
var command_send_buffer;

const lastest_command_required = new CustomEvent("lastest_command_required", {
});

const usr_command_issued = new CustomEvent("usr_command_issued", {
});

export function get_width(){ 
    return width;
}
export function get_height(){
    return height;
}
export function set_width(this_width){

    width=this_width;
}
export function set_height(this_height){
    height=this_height;
}



export function request_lastest_command(){
    document.dispatchEvent(lastest_command_required);
}

export function set_lastest_command(command){
    command_receive_buffer = command;
}

export function get_lastest_command(){
    return command_receive_buffer;
}



export function issue_command(){
    document.dispatchEvent(usr_command_issued);
}

export function set_command_send_buffer(command){
    command_send_buffer = command;
}

export function get_command_send_buffer(){
    return command_send_buffer;
}