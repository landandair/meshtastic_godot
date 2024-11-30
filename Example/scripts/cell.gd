extends Button
signal cell_updated(cell)

var main : Control

var cell_value : String = ""

@onready var background = $Background
@onready var border = $Border

func _ready():
	self_modulate.a = 0

func draw_x():
	var tween = get_tree().create_tween()
	self_modulate = Color("#00ffff")
	self_modulate.a = 0
	text = "X"
	cell_value = "X"
	tween.tween_property(self, "self_modulate:a", 1, 0.5)

func draw_o():
	var tween = get_tree().create_tween()
	self_modulate = Color("#ff4200")
	self_modulate.a = 0
	text = "O"
	cell_value = "O"
	tween.tween_property(self, "self_modulate:a", 1, 0.5)

func draw_cell():
	if main.is_game_end: return
	
	if cell_value == "":
		match main.turn:
			0:
				main.turn = 1
				draw_x()
			1:
				main.turn = 0
				draw_o()
	
	mouse_default_cursor_shape = Control.CURSOR_ARROW
	cell_updated.emit(self)

func glow(color):
	var tween = get_tree().create_tween()
	background.modulate = color
	background.modulate.a = 0
	tween.tween_property(background, "modulate:a", 1, 0.5)
