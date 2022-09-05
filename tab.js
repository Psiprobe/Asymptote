var width;
var height;

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
    console.log(get_height());
}