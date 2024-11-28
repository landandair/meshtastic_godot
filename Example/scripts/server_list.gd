extends HBoxContainer

@onready var short_name = $name
@onready var port = $port
signal joinGame(ip, subport)

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	pass # Replace with function body.


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass


func _on_button_pressed() -> void:
	joinGame.emit(short_name.text, int(port.text))
