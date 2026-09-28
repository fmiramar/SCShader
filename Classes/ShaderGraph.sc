ShaderGraph : Object {
    var <server, <nodes, <edges, <feedbackEdges, <isFreed;

    *new { |server|
        ^super.new.init(server ? ShaderServer.default)
    }

    init { |argServer|
        server = argServer;
        nodes = List.new;
        edges = List.new;
        feedbackEdges = List.new;
        isFreed = false;
        ^this
    }

    add { |shader|
        this.checkShader(shader);
        if(nodes.includes(shader).not) { nodes.add(shader) };
        ^this
    }

    remove { |shader|
        nodes.remove(shader);
        edges = edges.reject { |edge| edge[0] === shader or: { edge[2] === shader } };
        feedbackEdges = feedbackEdges.reject { |edge| edge[0] === shader };
        ^this
    }

    connect { |from, output = \out, to, input = \source|
        var edge;
        this.checkShader(from);
        this.checkShader(to);
        if(output.asSymbol != \out or: { input.asSymbol != \source }) {
            Error("ShaderGraph currently supports only \\out to \\source edges.").throw;
        };
        if(from === to) { Error("ShaderGraph cycles require connectFeedback.").throw };
        this.add(from).add(to);
        edge = [from, output.asSymbol, to, input.asSymbol];
        if(edges.any { |existing| existing[0] === from or: { existing[2] === to } }) {
            Error("The current ShaderGraph renderer supports one input and one output per pass.").throw;
        };
        if(this.pathExists(to, from)) { Error("ShaderGraph cycles require connectFeedback.").throw };
        edges.add(edge);
        ^this
    }

    connectFeedback { |shader, amount = 0.98|
        var order;
        this.checkShader(shader);
        if(amount.isNumber.not or: { amount.isNaN or: { amount < 0 or: { amount > 1 } } }) {
            Error("ShaderGraph feedback amount must be between 0 and 1.").throw;
        };
        order = this.validate;
        if(shader !== order.last) {
            Error("ShaderGraph feedback must target the terminal pass in this renderer milestone.").throw;
        };
        feedbackEdges = feedbackEdges.reject { |edge| edge[0] === shader };
        feedbackEdges.add([shader, amount.asFloat]);
        shader.feedback(\previous, amount);
        ^this
    }

    validate {
        if(isFreed) { Error("Cannot validate a freed ShaderGraph.").throw };
        if(nodes.isEmpty) { Error("ShaderGraph needs at least one Shader.").throw };
        nodes.do { |shader| this.checkShader(shader) };
        ^this.renderOrder
    }

    renderOrder {
        var heads = nodes.select { |node| edges.any { |edge| edge[2] === node }.not };
        var order = List.new, node;
        if(nodes.size == 1) { ^[nodes[0]] };
        if(heads.size != 1 or: { edges.size != (nodes.size - 1) }) {
            Error("ShaderGraph must be one connected acyclic pass chain in this renderer milestone.").throw;
        };
        node = heads[0];
        while { node.notNil } {
            order.add(node);
            node = edges.detect { |edge| edge[0] === node }.tryPerform(\at, 2);
        };
        if(order.size != nodes.size) { Error("ShaderGraph has a disconnected cycle or branch.").throw };
        ^order.asArray
    }

    commit {
        var order = this.validate;
        server.sendMsg('/scshader/v1/graph/set', *order.collect(_.id));
        ^order
    }

    clear {
        if(isFreed.not and: { server.isRunning }) { server.sendMsg('/scshader/v1/graph/set') };
        nodes.clear;
        edges.clear;
        feedbackEdges.clear;
        ^this
    }

    free {
        if(isFreed.not) {
            this.clear;
            isFreed = true;
        };
        ^this
    }

    pathExists { |from, target, visited|
        var seen = visited ? IdentitySet.new;
        if(from === target) { ^true };
        if(seen.includes(from)) { ^false };
        seen.add(from);
        ^edges.any { |edge|
            (edge[0] === from) and: { this.pathExists(edge[2], target, seen) }
        }
    }

    checkShader { |shader|
        if(isFreed) { Error("Cannot change a freed ShaderGraph.").throw };
        if(shader.isKindOf(Shader).not) { Error("ShaderGraph nodes must be Shaders.").throw };
        if(shader.server !== server) { Error("All ShaderGraph nodes must share one ShaderServer.").throw };
        if(shader.isFreed) { Error("ShaderGraph cannot contain a freed Shader.").throw };
        ^shader
    }
}
