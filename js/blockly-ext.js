// LOL https://stackoverflow.com/questions/57512741/prevent-a-new-block-from-being-attached-to-the-same-type-if-the-target-is-used-a
window.Blockly.Block.prototype.getMatchingConnection = function(otherBlock, conn) {
  var connections = this.getConnections_(true);
  var otherConnections = otherBlock.getConnections_(true);
  // if (connections.length !== otherConnections.length) {
  //   throw Error("Connection lists did not match in length.");
  // }
  for (var i = 0; i < otherConnections.length; i++) {
    if (otherConnections[i] === conn) {
      return connections[i];
    }
  }
  return null;
};

Blockly.Extensions.register(
    'allow_chain_in_thresh',
    function() {
        var thisBlock = this;
        this.setOnChange(function(changeEvent) {
            var parent = thisBlock.getSurroundParent();

            if (changeEvent.type === 'create') {
                if (!parent || parent.type !== 'thresh') {
                    thisBlock.setNextStatement(true, 'AAAA');
                }
            }

            if (changeEvent.blockId !== thisBlock.id || changeEvent.type !== 'move') {
                return;
            }

            // console.log(thisBlock.type, changeEvent.newParentId);
            var parent = thisBlock.getSurroundParent();
            if (changeEvent.newParentId && parent && parent.type === 'thresh') {
                thisBlock.setNextStatement(true, 'Policy');
            } else {
                if (thisBlock.nextConnection && thisBlock.nextConnection.isConnected()) {
                    thisBlock.nextConnection.disconnect();
                }
                // if (thisBlock.previousConnection && thisBlock.previousConnection.isConnected()) {
                //     thisBlock.previousConnection.disconnect();
                // }
                thisBlock.setNextStatement(true, 'AAAA');
            }
        });
    });

Blockly.Extensions.register('dynamic_options',
  function() {
    var dynamicOptions = function() {
        return window.BlocklyExt.dropdownCb();
    }
    var dropdown = new Blockly.FieldDropdown(dynamicOptions);
    this.inputList[0].removeField('Key');
    this.inputList[0].appendField(dropdown, 'Key')
  });

window.BlocklyExt = {};
window.BlocklyExt.initJs = function(workspace, compiled_cb, dropdown_cb) {
    function myUpdateFunction(event) {
        compiled_cb(Blockly.JavaScript.workspaceToCode(workspace));
    }
    workspace.addChangeListener(myUpdateFunction);
    workspace.addChangeListener(Blockly.Events.disableOrphans);

    window.BlocklyExt.dropdownCb = dropdown_cb;

    const onresize = function(e) {
      // Compute the absolute coordinates and dimensions of blocklyArea.
      var element = blocklyArea;
      let x = 0;
      let y = 0;
      do {
        x += element.offsetLeft;
        y += element.offsetTop;
        element = element.offsetParent;
      } while (element);
      // Position blocklyDiv over blocklyArea.
      blocklyDiv.style.left = x + 'px';
      blocklyDiv.style.top = y + 'px';
      blocklyDiv.style.width = blocklyArea.offsetWidth + 'px';
      blocklyDiv.style.height = blocklyArea.offsetHeight + 'px';
      Blockly.svgResize(workspace);
    };
    window.addEventListener('resize', onresize, false);
    onresize();
}
window.BlocklyExt.insertBegin = function(workspace, compiled_cb, dropdown_cb) {
    if (workspace.getTopBlocks().length > 0) {
        return;
    }

    var beginBlock = workspace.newBlock('begin');
    beginBlock.setDeletable(false);
    beginBlock.setEditable(false);
    beginBlock.moveBy(20, 20);
    beginBlock.initSvg();
    beginBlock.render();
}

Blockly.JavaScript.INDENT = '';
Blockly.JavaScript['begin'] = function(block) {
  return '';
};

Blockly.JavaScript['pk'] = function(block) {
    if (!block.getParent()) {
        return '';
    }

  var value_pk = Blockly.JavaScript.valueToCode(block, 'Key', Blockly.JavaScript.ORDER_ATOMIC);
    if (value_pk == '') {
        value_pk = '()';
    }

  var code = 'pk' + value_pk;
  return code;
};

Blockly.JavaScript['key'] = function(block) {
    if (!block.getParent()) {
        return ['', Blockly.JavaScript.ORDER_NONE];
    }

  var text_key = block.getFieldValue('Key');
  var code = text_key;
  // TODO: Change ORDER_NONE to the correct strength.
  return [code, Blockly.JavaScript.ORDER_NONE];
};

Blockly.JavaScript['my_key'] = function(block) {
    if (!block.getParent()) {
        return ['', Blockly.JavaScript.ORDER_NONE];
    }

  // TODO: Change ORDER_NONE to the correct strength.
  return ['_MY_KEY', Blockly.JavaScript.ORDER_NONE];
};

Blockly.JavaScript['thresh'] = function(block) {
  var number_threshold = block.getFieldValue('Threshold');
  var code = 'thresh(' + number_threshold + ',';
  var child = block.getChildren(true)[0];
  while (child) {
    code += Blockly.JavaScript[child.type](child);
    child = child.getNextBlock();
    if (child) {
      code += ',';
    }
  }
  code += ')';
  return code;
};

Blockly.JavaScript['older'] = function(block) {
    if (!block.getParent()) {
        return '';
    }

  var number_name = block.getFieldValue('value');
  var code = 'older(' + number_name + ')';
  return code;
};

Blockly.JavaScript['after'] = function(block) {
    if (!block.getParent()) {
        return '';
    }

  var number_name = block.getFieldValue('value');
  // TODO: Assemble JavaScript into code variable.
  var code = 'after(' + number_name + ')';
  return code;
};

Blockly.JavaScript['and'] = function(block) {
    if (!block.getParent()) {
        return '';
    }

  var statements_a = Blockly.JavaScript.statementToCode(block, 'A');
  var statements_b = Blockly.JavaScript.statementToCode(block, 'B');
  var code = 'and(' + statements_a + ',' + statements_b + ')';
  return code;
};

Blockly.JavaScript['or'] = function(block) {
    if (!block.getParent()) {
        return '';
    }

  var number_a_weight = block.getFieldValue('A_weight');
    if (number_a_weight == '1') {
        number_a_weight = '';
    } else {
        number_a_weight = number_a_weight + '@';
    }
  var statements_a = Blockly.JavaScript.statementToCode(block, 'A');
  var number_b_weight = block.getFieldValue('B_weight');
    if (number_b_weight == '1') {
        number_b_weight = '';
    } else {
        number_b_weight = number_b_weight + '@';
    }
  var statements_b = Blockly.JavaScript.statementToCode(block, 'B');
  var code = 'or(' + number_a_weight + statements_a + ',' + number_b_weight + statements_b + ')';
  return code;
};
